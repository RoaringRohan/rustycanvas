// server/state.rs

// This module manages the global canvas state. It supports:
//  - Storing the canvas state persistently using Sled key-value store

use sled::Db;
use std::sync::{Arc, RwLock};
use std::collections::VecDeque;
use serde::Serialize;

pub const CANVAS_WIDTH: u32 = 32;
pub const CANVAS_HEIGHT: u32 = 16;
pub const DEFAULT_COLOR: &str = "#000000";

#[derive(Clone, Serialize, Debug)]
pub struct PixelUpdate {
    pub x: u32,
    pub y: u32,
    pub color: String,
    pub timestamp: u64,
}

#[derive(Clone)] 
pub struct AppState {
    pub db: Db,
    pub history: Arc<RwLock<VecDeque<PixelUpdate>>>,
}


pub fn init_app_state(path: &str) -> AppState {
    // sled::open creates the database directory if it doesn't exist and recovers previous state if it does
    let db = sled::open(path).expect("Failed to open Sled database");

    AppState {
        db,
        history: Arc::new(RwLock::new(VecDeque::new())),
    }
}