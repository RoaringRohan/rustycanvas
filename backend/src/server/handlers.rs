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
use axum::extract::{State, Query};
use crate::server::state::{AppState, CANVAS_WIDTH, CANVAS_HEIGHT, DEFAULT_COLOR, PixelUpdate};
use std::time::{SystemTime, UNIX_EPOCH};

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

// Struct for getting updates since a timestamp
#[derive(Deserialize)]
pub struct GetUpdatesInput {
    pub since: u64, // Client sends the timestamp since they last synced
}

// Struct for updates response
#[derive(Serialize)]
pub struct UpdatesResponse {
    pub updates: Vec<PixelUpdate>,
    pub reset_required: bool, // Tell client if they are too far behind
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

// Logic to log a pixel update into history
pub fn log_pixel_update(state: &AppState, x: u32, y: u32, color: String) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let update = PixelUpdate { x, y, color, timestamp };

    if let Ok(mut history) = state.history.write() {
        history.push_back(update);
        if history.len() > 50 {
            history.pop_front();
        }
    }
}

// Logic to fetch updates since a given timestamp
pub fn fetch_updates_since(state: &AppState, since: u64) -> (Vec<PixelUpdate>, bool) {
    let history = state.history.read().unwrap();
    let mut updates = Vec::new();
    let mut reset_required = false;

    if let Some(first) = history.front() {
        // Only trigger reset if the buffer is full AND client is too old
        let buffer_limit_reached = history.len() >= 50;
        
        if buffer_limit_reached && since < first.timestamp {
             reset_required = true;
        } else {
             for item in history.iter() {
                if item.timestamp > since {
                    updates.push(item.clone());
                }
            }
        }
    }
    
    (updates, reset_required)
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
            // Log the update in history
            log_pixel_update(&app_state, payload.x, payload.y, payload.color);
            
            // Return Success Response
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

// GET /updates?since=123456789
pub async fn get_updates_handler(State(app_state): State<AppState>, Query(params): Query<GetUpdatesInput>) -> Json<UpdatesResponse> {
    let (updates, reset_required) = fetch_updates_since(&app_state, params.since);

    Json(UpdatesResponse {
        updates,
        reset_required,
    })
}
// -------------------------------- HANDLER FUNCTIONS ----------------------------------