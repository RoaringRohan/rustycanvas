// tests/integration_tests.rs

// Integration tests: testing the actual HTTP endpoints

use std::fs;
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use tower::util::ServiceExt; // for .oneshot()
use serde_json::json;
use backend::server::routes::create_router;
use backend::server::state::init_app_state;

// Test for GET /canvas endpoint
// Verifies that the full canvas is returned correctly
#[tokio::test]
async fn test_canvas_endpoint_returns_full_canvas() {
    // Sled creates a folder
    let test_db_path = "test_db_canvas_endpoint";
    
    // Clean up beforehand
    let _ = fs::remove_dir_all(test_db_path);

    let app_state = init_app_state(test_db_path);
    let app = create_router().with_state(app_state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/canvas")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = to_bytes(response.into_body(), 1_048_576).await.unwrap();
    let json_body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    // Verify dimensions (Sled logic should default to 32x16)
    assert_eq!(json_body["width"], 32);
    assert_eq!(json_body["height"], 16);

    let _ = fs::remove_dir_all(test_db_path);
}

// Test for POST /pixel endpoint
// Updates a pixel and verifies the update via GET /canvas
#[tokio::test]
async fn test_post_pixel_updates_canvas() {
    let test_db_path = "test_db_pixel_update";
    let _ = fs::remove_dir_all(test_db_path);

    let app_state = init_app_state(test_db_path);
    let app = create_router().with_state(app_state);

    let payload = json!({
        "x": 0,
        "y": 0,
        "color": "#FF0000"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/pixel")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify update
    let response_canvas = app
        .oneshot(
            Request::builder()
                .uri("/canvas")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body_bytes = to_bytes(response_canvas.into_body(), 1_048_576).await.unwrap();
    let json_canvas: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json_canvas["pixels"][0][0], "#FF0000");

    let _ = fs::remove_dir_all(test_db_path);
}

// Test for POST /pixel with out-of-bounds coordinates
#[tokio::test]
async fn test_post_pixel_out_of_bounds() {
    let test_db_path = "test_db_pixel_oob";
    let _ = fs::remove_dir_all(test_db_path);

    let app_state = init_app_state(test_db_path);
    let app = create_router().with_state(app_state);

    let payload = json!({ "x": 999, "y": 999, "color": "#123456" });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/pixel")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let _ = fs::remove_dir_all(test_db_path);
}