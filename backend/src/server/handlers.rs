// server/handlers.rs

// This file defines the handler functions for the Axum web server

// For your knowledge
// A handler function processes an incoming HTTP request and generates a response asynchronously
// Each handler function defines what should be returned to the client when a specific route is accessed
// Handler functions are 'async' because all axum handlers must be async to handle requests concurrently
// 'impl IntoResponse' allows axum to automatically convert the return type into a proper HTTP response

// server/handlers.rs

// This file defines the handler functions for the Axum web server

use axum::response::{IntoResponse, Json};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use axum::extract::State;
use crate::server::state::{AppState, CANVAS_WIDTH, CANVAS_HEIGHT, DEFAULT_COLOR}; 

// Struct for JSON response for canvas state
#[derive(serde::Serialize)]
pub struct CanvasResponse {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Vec<String>>,
}

// Struct for JSON input for pixel update
#[derive(Deserialize)]
pub struct PixelUpdateInput {
    pub x: u32,
    pub y: u32,
    pub color: String,
}

// Struct for JSON response for pixel update
#[derive(Serialize)]
pub struct PixelUpdateResponse {
    pub success: bool,
    pub error: Option<String>,
}

// Struct for clearing canvas response
#[derive(Serialize)]
pub struct ClearCanvasResponse {
    pub success: bool,
    pub message: String,
}

// -------------------------------- LOGIC FUNCTIONS ----------------------------------
// These functions contain the "Business Logic"

// Helper to generate a standardized key for the DB, e.g., "5:10"
fn make_key(x: u32, y: u32) -> String {
    format!("{}:{}", x, y)
}

// Logic to reconstruct the full 2D array from the Key-Value store
pub fn make_canvas_response(db: &sled::Db) -> CanvasResponse {
    let mut pixels = Vec::new();

    for y in 0..CANVAS_HEIGHT {
        let mut row = Vec::new();
        for x in 0..CANVAS_WIDTH {
            let key = make_key(x, y);
            
            // Try to get the pixel from DB. If not found, use DEFAULT_COLOR.
            let color = match db.get(&key) {
                Ok(Some(ivec)) => {
                    // Convert binary data back to String
                    String::from_utf8(ivec.to_vec()).unwrap_or(DEFAULT_COLOR.to_string())
                },
                _ => DEFAULT_COLOR.to_string(), // Default if key missing or error
            };
            row.push(color);
        }
        pixels.push(row);
    }

    CanvasResponse {
        width: CANVAS_WIDTH,
        height: CANVAS_HEIGHT,
        pixels,
    }
}

// Logic to update a single key-value pair in the DB
pub fn apply_pixel_update(db: &sled::Db, input: &PixelUpdateInput) -> Result<(), &'static str> {
    if input.x >= CANVAS_WIDTH || input.y >= CANVAS_HEIGHT {
        return Err("out_of_bounds");
    }

    let key = make_key(input.x, input.y);
    
    // Sled stores bytes, convert the hex string to bytes
    db.insert(&key, input.color.as_bytes())
        .map_err(|_| "db_write_error")?;

    db.flush().map_err(|_| "db_flush_error")?;

    Ok(())
}

// Logic to reset the canvas (clear the DB)
pub fn reset_canvas_db(db: &sled::Db) -> Result<(), &'static str> {
    // Sled's clear() removes all items from the Tree
    db.clear().map_err(|_| "db_clear_error")?;
    
    // Ensure the change is written to disk
    db.flush().map_err(|_| "db_flush_error")?;
    
    Ok(())
}
// -------------------------------- LOGIC FUNCTIONS ----------------------------------


// -------------------------------- HANDLER FUNCTIONS ----------------------------------
// These functions orchestrate the request:
// 1. Receive HTTP State/Input
// 2. Call Logic Functions
// 3. Handle Side Effects (Saving)
// 4. Return HTTP Response

// GET /canvas
pub async fn get_canvas_handler(State(app_state): State<AppState>) -> Json<CanvasResponse> {
    // Using logic function
    let response = make_canvas_response(&app_state.db);

    Json(response)
}

// POST /pixel
pub async fn update_pixel_handler(State(app_state): State<AppState>, Json(payload): Json<PixelUpdateInput>) -> (StatusCode, Json<PixelUpdateResponse>) {
    match apply_pixel_update(&app_state.db, &payload) {
        Ok(_) => {
            let response = PixelUpdateResponse {
                success: true,
                error: None,
            };
            (StatusCode::OK, Json(response))
        },
        Err(err_msg) => {
            let response = PixelUpdateResponse {
                success: false,
                error: Some(err_msg.to_string()),
            };
            (StatusCode::BAD_REQUEST, Json(response))
        }
    }
}

// POST /reset
pub async fn reset_canvas_handler(State(app_state): State<AppState>) -> (StatusCode, Json<ClearCanvasResponse>) {
    match reset_canvas_db(&app_state.db) {
        Ok(_) => {
            let response = ClearCanvasResponse {
                success: true,
                message: "Canvas reset successfully".to_string(),
            };
            (StatusCode::OK, Json(response))
        },
        Err(err_msg) => {
            let response = ClearCanvasResponse {
                success: false,
                message: err_msg.to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}
// -------------------------------- HANDLER FUNCTIONS ----------------------------------