// server/state.rs

// This module manages the global canvas state. It supports:
//   - Creating a blank canvas
//   - Loading the canvas from a JSON file
//   - Saving the canvas to a JSON file
//   - Sharing the canvas across the entire Axum server using
//     Arc<RwLock<T>> for safe concurrent reads/writes.

use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::Path,
    sync::Arc,
};
use tokio::sync::RwLock;

pub const CANVAS_FILE_PATH: &str = "data/canvas.json";

// Type alias for global shared state
pub type SharedCanvas = Arc<RwLock<CanvasState>>;

// Canvas struct
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CanvasState {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Vec<String>>, // 2D array of colors (#RRGGBB)
}

// Create a blank canvas (used on first run or reset)
pub fn create_blank_canvas(width: u32, height: u32) -> CanvasState {
    let row = vec!["#000000".to_string(); width as usize];
    let pixels = vec![row; height as usize];

    CanvasState {
        width,
        height,
        pixels,
    }
}

// Load canvas from a local JSON file
// If the file doesn't exist, create a blank canvas.
pub fn load_canvas_from_file() -> CanvasState {
    if Path::new(CANVAS_FILE_PATH).exists() {
        let file_contents = fs::read_to_string(CANVAS_FILE_PATH)
            .expect("Failed to read canvas.json file");
        serde_json::from_str(&file_contents)
            .expect("Failed to parse canvas.json")
    } else {
        // If the file doesn't exist, create a blank 32x32 canvas
        let canvas = create_blank_canvas(32, 32);

        // Save it for future runs
        save_canvas_to_file(&canvas);

        canvas
    }
}

// Save canvas to local JSON file
// THIS LOGIC WILL CHANGE ONCE WE SWITCH TO THE CLOUD STORAGE SOLUTION
pub fn save_canvas_to_file(canvas: &CanvasState) {
    let json = serde_json::to_string_pretty(canvas)
        .expect("Failed to serialize CanvasState");

    // Ensure "data" directory exists
    if !Path::new("data").exists() {
        fs::create_dir("data").expect("Failed to create data directory");
    }

    fs::write(CANVAS_FILE_PATH, json)
        .expect("Failed to write canvas.json");
}

// Initialize a SharedCanvas (wrapped in Arc<RwLock>)
pub fn init_shared_canvas() -> SharedCanvas {
    let canvas = load_canvas_from_file();
    Arc::new(RwLock::new(canvas))
}