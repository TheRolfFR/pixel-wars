use actix_web::{error, get, web, HttpResponse, Responder};
use serde::Serialize;
use crate::model::{self, BackendError};
use redis::AsyncCommands;
use base64::prelude::*;

#[derive(Debug, Serialize)]
struct CanvasInfoSize {
    width: usize,
    height: usize,
}

#[derive(Debug, Serialize)]
struct CanvasInfoResponse {
    canvas: String,
    size: CanvasInfoSize
}

pub async fn canvas_get(
    app_state: web::Data<model::BackendAppState>,
    redis: web::Data<redis::Client>,
    config: web::Data<model::Config>
) -> actix_web::Result<impl Responder> {
    let mut canvas = app_state.canvas_valid.lock().unwrap();

    if !canvas.valid {
        let mut con = redis.get_multiplexed_async_connection().await.map_err(BackendError::from)?;
        let colors: Vec<u8> = con.get("canvas").await.map_err(BackendError::from)?;

        (*canvas).colors = colors;
        (*canvas).valid = true;
    }

    Ok(HttpResponse::Ok().json(CanvasInfoResponse {
        canvas: BASE64_STANDARD.encode(&canvas.colors),
        size: CanvasInfoSize {
            width: config.canvas_width,
            height: config.canvas_height
        }
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