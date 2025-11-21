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
use crate::server::state::{AppState, CanvasState, save_canvas_to_file}; 

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

// -------------------------------- LOGIC FUNCTIONS ----------------------------------
// These functions contain the "Business Logic". 
// They do NOT know about HTTP, Axum, or Files. They just calculate data.

// Logic to transform internal state into the public response struct
pub fn make_canvas_response(canvas: &CanvasState) -> CanvasResponse {
    CanvasResponse {
        width: canvas.width,
        height: canvas.height,
        pixels: canvas.pixels.clone(),
    }
}

// Logic to validate and apply a pixel update
pub fn apply_pixel_update(canvas: &mut CanvasState, input: &PixelUpdateInput) -> Result<(), &'static str> {
    if input.x >= canvas.width || input.y >= canvas.height {
        return Err("out_of_bounds");
    }

    let x = input.x as usize;
    let y = input.y as usize;

    canvas.pixels[y][x] = input.color.clone();
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
    let canvas_read = app_state.canvas.read().await;
    
    // Using logic function
    let response = make_canvas_response(&canvas_read);

    Json(response)
}

// POST /pixel
pub async fn update_pixel_handler(
    State(app_state): State<AppState>, 
    Json(payload): Json<PixelUpdateInput>
) -> (StatusCode, Json<PixelUpdateResponse>) {
    
    let mut canvas_write = app_state.canvas.write().await;

    // Using logic function
    match apply_pixel_update(&mut canvas_write, &payload) {
        Ok(_) => {
            // If logic succeeded, we handle the side effect (saving)
            save_canvas_to_file(&canvas_write, &app_state.file_path);

            let response = PixelUpdateResponse {
                success: true,
                error: None,
            };
            (StatusCode::OK, Json(response))
        },
        Err(err_msg) => {
            // If logic failed (e.g. out of bounds), we format the error for HTTP
            let response = PixelUpdateResponse {
                success: false,
                error: Some(err_msg.to_string()), // Convert "out_of_bounds" to string
            };
            // We map the error to a 400 Bad Request
            (StatusCode::BAD_REQUEST, Json(response))
        }
    }
}
// -------------------------------- HANDLER FUNCTIONS ----------------------------------


// *******************************************************************************************************************************
//-------------------------------- TEMPLATE CODE -------------------------------------
// Used just to illustrate our structure, you can test around with this endpoints, they don't affect the canvas logic
#[derive(Serialize)]
pub struct TestGetResponse {
    pub status: String,
    pub message: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TestPostInput {
    pub username: String,
    pub id: u32
}

#[derive(Serialize)]
pub struct TestPostResponse {
    pub received: bool,
    pub echo: TestPostInput
}

pub fn make_test_get_response() -> TestGetResponse {
    TestGetResponse {
        status: "ok".to_string(),
        message: "The test get endpoint handler is working!".to_string(),
    }
}

pub fn make_test_post_response(input: TestPostInput) -> TestPostResponse {
    TestPostResponse {
        received: true,
        echo: input,
    }
}

pub async fn root_handler() -> impl IntoResponse {
    "Welcome to the root handler!"
}

pub async fn test_get_handler() -> impl IntoResponse {
    let response = make_test_get_response();
    Json(response)
}

pub async fn test_post_handler(Json(payload): Json<TestPostInput>) -> impl IntoResponse {
    let response = make_test_post_response(payload);
    Json(response)
}
// *******************************************************************************************************************************