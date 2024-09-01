use std::{mem::size_of, vec};

use actix_web::{error, get, web, HttpResponse, Responder};
use serde::Serialize;
use crate::model::{self, BackendError};
use redis::{AsyncCommands, Commands, RedisError};
use base64::prelude::*;

#[derive(Debug, Serialize)]
struct CanvasInfoSize {
    width: u16,
    height: u16,
}

#[derive(Debug, Serialize)]
struct CanvasInfoResponse {
    canvas: String,
    size: CanvasInfoSize,
    colors: Vec<[u8; 3]>
}

pub struct CanvasChunk;
impl CanvasChunk {
    fn chunk_index_to_key(chunk_index_x: usize, chunk_index_y: usize) -> String {
        format!("{}_{}_{}", model::CANVAS_DB_KEY, chunk_index_x, chunk_index_y)
    }

    async fn chunk_create(config: &model::Config, con: &mut impl AsyncCommands, chunk_key: &str) -> Result<Vec<u8>, RedisError> {
        let vec_size = (config.canvas_chunk_size as usize * config.canvas_chunk_size as usize) / 2 * size_of::<u16>();
        con.setbit(chunk_key, vec_size * 8 - 1, false).await?; // set latest chunk bit (thus *8 - 1) to create empty string with 0 value
        Ok(vec![0; vec_size])
    }

    async fn chunk_get(config: &model::Config, con: &mut impl AsyncCommands, chunk_index_x: usize, chunk_index_y: usize) -> Result<Vec<u8>, RedisError> {
        let chunk_key = Self::chunk_index_to_key(chunk_index_x, chunk_index_y);
        let opt_colors: Option<Vec<u8>> = con.get(&chunk_key).await?;

        Ok(match opt_colors {
            Some(colors) => colors,
            None => Self::chunk_create(config, con, &chunk_key).await?
        })
    }

    fn two_dim(one_dim: Vec<u8>, slice_width: usize, slice_height: usize) -> Vec<Vec<u8>> {
        let mut result = vec![];
        let mut iter = one_dim.iter();
        for _ in 0..slice_height {
            let mut row = vec![];
            for _ in 0..slice_width {
                row.push(*iter.next().unwrap());
            }
            result.push(row);
        }

        result
    }

    fn one_dim(two_dim: Vec<Vec<u8>>) -> Vec<u8> {
        two_dim.into_iter().flatten().collect()
    }

    fn merge_two_dim_rows(mut two_dim_vec: Vec<Vec<Vec<u8>>>) -> Vec<Vec<u8>> {
        let final_rows = two_dim_vec[0].len();
        let mut result = vec![vec![]; final_rows];

        for chunk in two_dim_vec.iter_mut() {
            for (row_index, row) in chunk.iter_mut().enumerate() {
                result[row_index].append(row);
            }
        }

        result
    }

    fn chunk_update(
        config: &model::Config,
        con: &mut impl Commands,
        chunk_loc: model::ChunkLocation,
        pixel_color: &model::PixelColorUpdateMessageColor
    ) -> Result<(), RedisError> {
        let (chunk_index, chunk_pos) = chunk_loc;
        let (chunk_index_x, chunk_index_y) = chunk_index;
        let (chunk_pos_x, chunk_pos_y) = chunk_pos;

        let chunk_key = Self::chunk_index_to_key(chunk_index_x, chunk_index_y);

        // bit offset because multiplied by 4 where 4 = 8 / pixels_per_byte
        let pixel_bit_width = 8 / config.pixels_per_bytes;
        let bit_offset = (chunk_pos_y * config.canvas_chunk_size as usize + chunk_pos_x) as usize * pixel_bit_width;

        for i in 0..4usize {
            let is_bit_one = (pixel_color & (1 << i)) > 0;

            let redis_offset = bit_offset + (3-i);

            con.setbit(&chunk_key, redis_offset, is_bit_one)?;
        }

        Ok(())
    }
}

async fn canvas_redis_get(
    redis: &redis::Client,
    config: &model::Config
) -> Result<Vec<u8>, RedisError>
{
    let mut con = redis.get_multiplexed_async_connection().await?;

    let mut result = vec![];

    let (chunk_cols, chunk_rows) = config.canvas_chunks();
    for chunk_index_y in 0..chunk_rows {
        let mut row_chunks = vec![];
        for chunk_index_x in 0..chunk_cols {
            let one_dim_chunk = CanvasChunk::chunk_get(config, &mut con, chunk_index_x, chunk_index_y).await?;

            let slice_width  = config.canvas_width  as usize - chunk_index_x * config.canvas_chunk_size as usize;
            let slice_height = config.canvas_height as usize - chunk_index_y * config.canvas_chunk_size as usize;

            let two_dim_chunk = CanvasChunk::two_dim(one_dim_chunk, slice_width, slice_height);
            row_chunks.push(two_dim_chunk);
        }

        result.append(&mut CanvasChunk::merge_two_dim_rows(row_chunks));
    }

    Ok(CanvasChunk::one_dim(result))
}

pub fn canvas_redis_set(
    redis: &redis::Client,
    config: &model::Config,
    pixel_update: &model::PixelColorUpdateMessage
) -> Result<(), String>
{
    let chunk_loc = config.canvas_pos_to_chunk_location(pixel_update.pos_x, pixel_update.pos_y);
    let mut con = redis.get_connection()
        .map_err(|e| e.to_string())?;

    CanvasChunk::chunk_update(config, &mut con, chunk_loc, &pixel_update.color)
        .map_err(|e| e.to_string())
}

pub async fn canvas_get(
    redis: web::Data<redis::Client>,
    config: web::Data<model::Config>
) -> actix_web::Result<impl Responder> {
    let canvas = model::Canvas {
        colors: canvas_redis_get(&redis, &config).await.map_err(BackendError::from)?,
        valid: true
    };

    let encoded_canvas = BASE64_STANDARD.encode(&canvas.colors);

    let active_colors =  if let Some(colors_active) = &config.colors_active {
        let mut filtered_ordered_colors = Vec::with_capacity(colors_active.len());
        for color_index in colors_active {
            if let Some(color) = config.colors.get(*color_index) {
                filtered_ordered_colors.push(*color);
            }
        }
        filtered_ordered_colors
    } else {
        config.colors.clone()
    };

    Ok(HttpResponse::Ok().json(CanvasInfoResponse {
        canvas: encoded_canvas,
        size: CanvasInfoSize {
            width: config.canvas_width,
            height: config.canvas_height
        },
        colors: active_colors
    }))
}

#[derive(Debug, Serialize)]
struct CanvasUpdateResponse {
    offset: u32,
    color: u8
}

pub async fn canvas_update(
    app_state: web::Data<model::BackendAppState>,
    path: web::Path<(u32, u8)>
) -> actix_web::Result<impl Responder> {
    let mut canvas = app_state.canvas_valid.lock().unwrap();
    let (offset, color) = path.into_inner();

    (*canvas).valid = false;

    Ok(HttpResponse::Ok().json(CanvasUpdateResponse {
        offset,
        color
    }))
}
