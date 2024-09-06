use actix::prelude::Message;
use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use redis::RedisError;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json;
use std::fs::File;
use std::error::Error;
use std::io::BufReader;
use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;


const CANVAS_SIZE_DEFAULT: u16 = 256;
fn canvas_size_default() -> u16 { CANVAS_SIZE_DEFAULT }
const PIXELS_PER_BYTES_DEFAULT: usize = 2;
fn pixels_per_bytes_default() -> usize { PIXELS_PER_BYTES_DEFAULT }

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub redis_url: String,
    pub host: String,
    #[serde(default)]
    pub debug_mode: bool,

    #[serde(default = "canvas_size_default")]
    pub canvas_width: u16,
    #[serde(default = "canvas_size_default")]
    pub canvas_height: u16,
    #[serde(default = "canvas_size_default")]
    pub canvas_chunk_size: u16,

    pub base_pixel_amount: usize,
    #[serde(deserialize_with = "deserialize_duration_seconds")]
    pub timeout: Duration,
    #[serde(default = "pixels_per_bytes_default")]
    pub pixels_per_bytes: usize,

    pub colors: Vec<[u8; 3]>,
    pub colors_active: Option<Vec<usize>>,
}

fn deserialize_duration_seconds<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let seconds = u64::deserialize(deserializer)?;
    Ok(Duration::from_secs(seconds))
}

pub type ChunkLocation = ((usize, usize), (usize, usize));

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        // Open file in RO mode with buffer
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        // Read the JSON contents of the file
        let result: Config = serde_json::from_reader(reader)?;

        Ok(result)
    }
    fn chunk_number(&self, size: u16) -> usize {
        (size / self.canvas_chunk_size + if size % self.canvas_chunk_size != 0 { 1 } else { 0 }).into()
    }
    pub fn canvas_chunks(&self) -> (usize, usize) {
        (self.chunk_number(self.canvas_width), self.chunk_number(self.canvas_height))
    }
    pub fn canvas_pos_to_chunk_location(&self, pos_x: u16, pos_y: u16) -> ChunkLocation {
        let chunk_index = ((pos_x / self.canvas_chunk_size) as usize, (pos_y / self.canvas_chunk_size) as usize);
        let chunk_pos = ((pos_x % self.canvas_chunk_size) as usize, (pos_y % self.canvas_chunk_size) as usize);

        (chunk_index, chunk_pos)
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

pub type PixelColorUpdateMessageColor = u8;

#[derive(Debug, Serialize, Deserialize, Clone, Message)]
#[rtype(result = "()")]
pub struct PixelColorUpdateMessage {
    pub pos_x: u16,
    pub pos_y: u16,
    pub color: PixelColorUpdateMessageColor
}

#[derive(Debug, Serialize, Deserialize, Clone, Message)]
#[rtype(result = "Result<(), String>")]
pub struct UserPixelColorMessage {
    pub pixel_update: PixelColorUpdateMessage,
    pub uuid: String
}

pub const SESSION_COOKIE_NAME: &str = "sessionUUID";

pub const CANVAS_DB_KEY: &str = "canvas";
