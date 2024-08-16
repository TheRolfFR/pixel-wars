use actix::prelude::Message;
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
    pub redis_url: Option<String>,
    pub canvas_width: u16,
    pub canvas_height: u16,
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

impl BackendError {
    pub fn new(
        error: &'static str,
        details: String
    ) -> Self {
        Self { error, details }
    }
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
        log::error!("RedisError: {:?}", val);
        BackendError {
            error: "Error retrieving canvas from redis",
            details: val.to_string()
        }
    }
}
impl From<serde_json::Error> for BackendError {
    fn from(val: serde_json::Error) -> Self {
        log::error!("serde_json::Error: {:?}", val);
        BackendError {
            error: "Error with serialize/deserialize from serde_json",
            details: val.to_string()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Canvas {
    pub valid: bool,
    pub colors: Vec<u8>
}

impl Canvas {
    pub fn new(canvas_width: usize, canvas_height: usize) -> Self {
        Canvas {
            colors: vec![0; canvas_width as usize * canvas_height as usize / 2],
            valid: false
        }
    }
}

#[derive(Debug)]
pub struct BackendAppState {
    pub canvas_valid: Mutex<Canvas>
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub name: String,
    pub email: String,
    pub image: String
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Client {
    pub profile: Option<Profile>,
    pub last_timestamp: f64,
    pub remaining_pixels: usize
}

impl Client {
    pub fn encode_json(&self) -> Result<std::string::String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Message)]
#[rtype(result = "()")]
pub struct PixelColorUpdateMessage {
    pub pos_x: u16,
    pub pos_y: u16,
    pub color: u8
}

#[derive(Debug, Serialize, Deserialize, Clone, Message)]
#[rtype(result = "Result<(), String>")]
pub struct UserPixelColorMessage {
    pub pixel_update: PixelColorUpdateMessage,
    pub uuid: String
}

pub const SESSION_COOKIE_NAME: &str = "sessionUUID";
pub const CANVAS_DB_KEY: &str = "canvas";
