use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use redis::RedisError;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::error::Error;
use std::io::BufReader;
use std::path::Path;
use std::sync::Mutex;


#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub canvas_width: usize,
    pub canvas_height: usize,
    pub pixels_per_minute: usize,
    pub debug_mode: bool,
    pub host: String
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        // Open file in RO mode with buffer
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        // Read the JSON contents of the file
        let result: Config = serde_json::from_reader(reader)?;

        Ok(result)
    }
}

#[derive(Debug, Serialize)]
pub struct BackendError {
    pub error: &'static str,
    pub details: String
}

impl std::fmt::Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Backend error: {}    details: {}", self.error, self.details)
    }
}

impl ResponseError for BackendError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
            .json(self)
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl From<RedisError> for BackendError {
    fn from(val: RedisError) -> Self {
        BackendError {
            error: "Error retrieving canvas from redis",
            details: val.to_string()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Canvas {
    pub valid: bool,
    pub colors: Vec<u8>
}

#[derive(Debug)]
pub struct BackendAppState {
    pub canvas_valid: Mutex<Canvas>
}
