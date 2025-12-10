// server/state.rs

// This module manages the global canvas state. It supports:
//   - Creating a blank canvas
//   - Loading the canvas from a JSON file
//   - Saving the canvas to a JSON file
//   - Sharing the canvas across the entire Axum server using
//     Arc<RwLock<T>> for safe concurrent reads/writes.

use std::sync::Arc;
use sled::Db;

pub const CANVAS_WIDTH: u32 = 32;
pub const CANVAS_HEIGHT: u32 = 16;
pub const DEFAULT_COLOR: &str = "#000000";

#[derive(Clone)] 
pub struct AppState {
    pub db: Db,
}

pub fn init_app_state(path: &str) -> AppState {
    // sled::open creates the database directory if it doesn't exist and recovers previous state if it does
    let db = sled::open(path).expect("Failed to open Sled database");

    AppState {
        db
    }
}