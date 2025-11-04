// server/handlers.rs

// This file defines the handler functions for the Axum web server

// For your knowledge
// A handler function processes an incoming HTTP request and generates a response asynchronously
// Each handler function defines what should be returned to the client when a specific route is accessed
// Handler functions are 'async' because all axum handlers must be async to handle requests concurrently
// 'impl IntoResponse' allows axum to automatically convert the return type into a proper HTTP response

use axum::response::{IntoResponse, Json};
use serde::{Deserialize, Serialize};

// Structs used for JSON I/O
// ───────────────────────────────

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

// Logic functions (unit-testable)
// ───────────────────────────────

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

// HTTP Handlers (Axum endpoints)
// ───────────────────────────────

// GET request made to "/"
pub async fn root_handler() -> impl IntoResponse {
    "Welcome to the root handler!"
}

// GET request made to "/test-get"
pub async fn test_get_handler() -> impl IntoResponse {
    let response = TestGetResponse {
        status: "ok".to_string(),
        message: "The test get endpoint handler is working!".to_string()
    };
    Json(response)
}

// POST request made to "/test-post"
pub async fn test_post_handler(Json(payload): Json<TestPostInput>) -> impl IntoResponse {
    let response = TestPostResponse {
        received: true,
        echo: payload
    };
    Json(response)
}