// server/handlers.rs

// This file defines the handler functions for the Axum web server

// For your knowledge
// A handler function processes an incoming HTTP request and generates a response asynchronously
// Each handler function defines what should be returned to the client when a specific route is accessed
// Handler functions are 'async' because all axum handlers must be async to handle requests concurrently
// 'impl IntoResponse' allows axum to automatically convert the return type into a proper HTTP response

use axum::response::{IntoResponse, Json};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use axum::extract::State;
use crate::server::state::{SharedCanvas, save_canvas_to_file};

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

// Logic-only pixel update function (used for unit tests)
pub fn apply_pixel_update(canvas: &mut crate::server::state::CanvasState, input: &crate::server::handlers::PixelUpdateInput) -> Result<(), &'static str> {
    if input.x >= canvas.width || input.y >= canvas.height {
        return Err("out_of_bounds");
    }

    let x = input.x as usize;
    let y = input.y as usize;

    canvas.pixels[y][x] = input.color.clone();
    Ok(())
}


// GET /canvas
// Returns full canvas JSON
pub async fn get_canvas_handler(
    State(canvas): State<SharedCanvas>) -> Json<CanvasResponse> {
    let canvas_read = canvas.read().await;

    let response = CanvasResponse {
        width: canvas_read.width,
        height: canvas_read.height,
        pixels: canvas_read.pixels.clone(),
    };

    Json(response)
}

// POST /pixel
// Updates a single pixel in the canvas
pub async fn update_pixel_handler(State(canvas): State<SharedCanvas>, Json(payload): Json<PixelUpdateInput>) -> (StatusCode, Json<PixelUpdateResponse>) {
    // Acquire a write lock since we're mutating the canvas
    let mut canvas_write = canvas.write().await;

    // Validate coordinates: x in [0, width), y in [0, height)
    if payload.x >= canvas_write.width || payload.y >= canvas_write.height {
        let response = PixelUpdateResponse {
            success: false,
            error: Some("Pixel coordinates out of bounds".to_string()),
        };
        return (StatusCode::BAD_REQUEST, Json(response));
    }

    // Safe to index now
    let x = payload.x as usize;
    let y = payload.y as usize;

    // Update the pixel color
    canvas_write.pixels[y][x] = payload.color.clone();

    // Persist the updated canvas to disk
    save_canvas_to_file(&canvas_write);

    // Return success
    let response = PixelUpdateResponse {
        success: true,
        error: None,
    };
    (StatusCode::OK, Json(response))
}

// *******************************************************************************************************************************
//-------------------------------- TEMPLATE CODE -------------------------------------
// Struct for JSON response for test-get
#[derive(Serialize)]
pub struct TestGetResponse {
    pub status: String,
    pub message: String
}

// Struct for JSON input for test-post
#[derive(Serialize, Deserialize, Clone)]
pub struct TestPostInput {
    pub username: String,
    pub id: u32
}

// Struct for JSON response for test-post
#[derive(Serialize)]
pub struct TestPostResponse {
    pub received: bool,
    pub echo: TestPostInput
}
//-------------------------------- TEMPLATE CODE -------------------------------------

// -------------------------------- TEMPLATE LOGIC FUNCTIONS (USED FOR UNIT TESTS) ----------------------------------
// Constructs the JSON response for the GET test endpoint.
pub fn make_test_get_response() -> TestGetResponse {
    TestGetResponse {
        status: "ok".to_string(),
        message: "The test get endpoint handler is working!".to_string(),
    }
}

// Constructs the JSON response for the POST test endpoint.
pub fn make_test_post_response(input: TestPostInput) -> TestPostResponse {
    TestPostResponse {
        received: true,
        echo: input,
    }
}
// -------------------------------- TEMPLATE LOGIC FUNCTIONS (USED FOR UNIT TESTS) ----------------------------------

// -------------------------------- TEMPLATE HANDLER FUNCTIONS ----------------------------------
// GET request made to "/"
pub async fn root_handler() -> impl IntoResponse {
    "Welcome to the root handler!"
}

// GET request made to "/test-get"
pub async fn test_get_handler() -> impl IntoResponse {
    let response = make_test_get_response();
    Json(response)
}

// POST request made to "/test-post"
pub async fn test_post_handler(Json(payload): Json<TestPostInput>) -> impl IntoResponse {
    let response = make_test_post_response(payload);
    Json(response)
}
// -------------------------------- TEMPLATE HANDLER FUNCTIONS ----------------------------------
// *******************************************************************************************************************************