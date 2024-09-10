use std::{mem::size_of, vec};

use actix_web::{error, get, web, HttpResponse, Responder};
use serde::Serialize;
use crate::model::{self, BackendError, ConfigColor};
use redis::{AsyncCommands, Commands, RedisError};
use base64::prelude::*;

pub struct CanvasChunk;
impl CanvasChunk {
    fn chunk_index_to_key(chunk_index_x: usize, chunk_index_y: usize) -> String {
        let result = format!("{}_{}_{}", model::CANVAS_DB_KEY, chunk_index_x, chunk_index_y);
        result
    }

    async fn chunk_create(config: &model::Config, con: &mut impl AsyncCommands, chunk_key: &str) -> Result<Vec<u8>, RedisError> {
        let vec_size = (config.canvas_chunk_size as usize * config.canvas_chunk_size as usize) / config.pixels_per_bytes;
        con.setbit(chunk_key, vec_size * 8 - 1, false).await?; // set latest chunk bit (thus *8 - 1) to create empty string with 0 value
        Ok(vec![0; vec_size])
    }

    async fn chunk_get(config: &model::Config, con: &mut impl AsyncCommands, chunk_index_x: usize, chunk_index_y: usize) -> Result<Vec<u8>, RedisError> {
        let chunk_key = Self::chunk_index_to_key(chunk_index_x, chunk_index_y);
        let opt_colors: Option<Vec<u8>> = con.get(&chunk_key).await?;

        let mut colors = match opt_colors {
            Some(colors) => colors,
            None => Self::chunk_create(config, con, &chunk_key).await?
        };

        colors.truncate((config.canvas_chunk_size as usize * config.canvas_chunk_size as usize) / config.pixels_per_bytes);
        Ok(colors)
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
) -> Result<Vec<Vec<Vec<u8>>>, RedisError>
{
    let mut con = redis.get_multiplexed_async_connection().await?;

    let chunk_numbers = config.canvas_chunks();
    let (chunk_rows, chunk_cols) = chunk_numbers;

    let mut result = Vec::with_capacity(chunk_rows);

    for index_x in 0..chunk_rows {
        let mut row_vec = Vec::with_capacity(chunk_cols);
        for index_y in 0..chunk_cols {
            row_vec.push(CanvasChunk::chunk_get(config, &mut con, index_x, index_y).await?);
        }
        result.push(row_vec);
    }

    return Ok(result);
}

pub fn canvas_redis_set(
    redis: &redis::Client,
    config: &model::Config,
    pixel_update: &model::PixelColorUpdateMessage
) -> Result<(), String>
{
    let chunk_loc = config.canvas_pos_to_chunk_location(pixel_update.pos_x.into(), pixel_update.pos_y.into());
    let mut con = redis.get_connection()
        .map_err(|e| e.to_string())?;

    CanvasChunk::chunk_update(config, &mut con, chunk_loc, &pixel_update.color)
        .map_err(|e| e.to_string())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CanvasInfoSize {
    width: usize,
    height: usize,
    chunk_size: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CanvasInfoResponse {
    canvas: Vec<Vec<String>>, // list of chunks
    size: CanvasInfoSize,
    colors: Vec<ConfigColor>,
}

#[get("/canvas")]
pub async fn canvas_get(
    redis: web::Data<redis::Client>,
    config: web::Data<model::Config>
) -> actix_web::Result<impl Responder> {
    let canvas_chunks = canvas_redis_get(&redis, &config).await.map_err(BackendError::from)?;

    let encoded_chunks = canvas_chunks.into_iter().map(|chunk_row|
        chunk_row.into_iter().map(|chunk| BASE64_STANDARD.encode(&chunk)).collect::<Vec<_>>()
    ).collect::<Vec<_>>();

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
        canvas: encoded_chunks,
        size: CanvasInfoSize {
            width: config.canvas_width,
            height: config.canvas_height,
            chunk_size: config.canvas_chunk_size,
        },
        colors: active_colors
    }))
}
