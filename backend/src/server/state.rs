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

// Type alias for global shared state
// Arc is Atomic Reference Counted pointer for thread-safe shared ownership
// RwLock allows multiple concurrent reads or exclusive write access
pub type SharedCanvas = Arc<RwLock<CanvasState>>;

// This struct holds everything our handlers need to know
#[derive(Clone)] 
pub struct AppState {
    pub canvas: SharedCanvas,
    pub file_path: String,
}

// Canvas struct
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CanvasState {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Vec<String>> // 2D array of colors (#RRGGBB)
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

// Load canvas from a file path provided as an argument
pub fn load_canvas_from_file(path: &str) -> CanvasState {
    if Path::new(path).exists() {
        let file_contents = fs::read_to_string(path)
            .expect("Failed to read canvas file");
        serde_json::from_str(&file_contents)
            .expect("Failed to parse canvas json")
    } else {
        // If the file doesn't exist, create a blank 32x16 canvas
        let canvas = create_blank_canvas(32, 16); // Updated to match your json (32x16)
        
        // We attempt to save it immediately so the file is created
        save_canvas_to_file(&canvas, path);

        canvas
    }
}

// Save canvas to a specific file path
pub fn save_canvas_to_file(canvas: &CanvasState, path: &str) {
    let json = serde_json::to_string_pretty(canvas)
        .expect("Failed to serialize CanvasState");

    // We get the parent directory from the path (e.g. "data" from "data/canvas.json")
    if let Some(parent) = Path::new(path).parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).expect("Failed to create directory");
        }
    }

    fs::write(path, json)
        .expect("Failed to write canvas file");
}

// Initialize a SharedCanvas (wrapped in Arc<RwLock>)
pub fn init_app_state(path: &str) -> AppState {
    let canvas = load_canvas_from_file(path);
    let shared_canvas = Arc::new(RwLock::new(canvas));
    
    AppState {
        canvas: shared_canvas,
        file_path: path.to_string(),
    }
}