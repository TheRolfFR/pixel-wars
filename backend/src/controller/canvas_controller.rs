use std::mem::size_of;

use actix_web::{error, get, web, HttpResponse, Responder};
use serde::Serialize;
use crate::model::{self, BackendError};
use redis::{AsyncCommands, Commands};
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

async fn canvas_redis_get(
    redis: &redis::Client,
    config: &model::Config
) -> Result<Vec<u8>, BackendError>
{
    let mut con = redis.get_multiplexed_async_connection().await.map_err(BackendError::from)?;
    let opt_colors: Option<Vec<u8>> = con.get(model::CANVAS_DB_KEY).await.map_err(BackendError::from)?;

    match opt_colors {
        Some(colors) => Ok(colors),
        None => {
            let vec_size: usize = (config.canvas_width as usize) * (config.canvas_height as usize) / 2 * size_of::<u16>();
            con.setbit(model::CANVAS_DB_KEY, vec_size * 8 - 1, false).await.map_err(BackendError::from)?;
            Ok(model::Canvas::new(config.canvas_width as usize, config.canvas_height as usize).colors)
        }
    }
}

pub fn canvas_redis_set(
    redis: &redis::Client,
    config: &model::Config,
    pixel_update: &model::PixelColorUpdateMessage
) -> Result<(), String>
{
    let offset = (pixel_update.pos_x + config.canvas_width * pixel_update.pos_y) as usize * 4;

    let mut con = redis.get_connection()
        .map_err(|e| e.to_string())?;

    for i in 0..4usize {
        let bit = (pixel_update.color & (1 << i)) > 0;

        let redis_offset = offset + (3-i);

        con.setbit(model::CANVAS_DB_KEY, redis_offset, bit)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub async fn canvas_get(
    redis: web::Data<redis::Client>,
    config: web::Data<model::Config>
) -> actix_web::Result<impl Responder> {
    let canvas = model::Canvas {
        colors: canvas_redis_get(&redis, &config).await?,
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
