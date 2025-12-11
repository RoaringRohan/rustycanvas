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

// Test for POST /reset endpoint
#[tokio::test]
async fn test_reset_endpoint() {
    let test_db_path = "test_db_reset_endpoint";
    let _ = fs::remove_dir_all(test_db_path);

    let app_state = init_app_state(test_db_path);
    let app = create_router().with_state(app_state);

    // Paint a pixel (Red)
    let pixel_payload = json!({
        "x": 5,
        "y": 5,
        "color": "#FF0000"
    });
    
    // We reuse the app clone for the first request
    let _ = app.clone().oneshot(
        Request::builder()
            .uri("/pixel")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(Body::from(pixel_payload.to_string()))
            .unwrap(),
    ).await.unwrap();

    // Call /reset
    let response = app.clone().oneshot(
        Request::builder()
            .uri("/reset")
            .method("POST")
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify Canvas is Black
    let response_canvas = app.oneshot(
        Request::builder()
            .uri("/canvas")
            .method("GET")
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();

    let body_bytes = to_bytes(response_canvas.into_body(), 1_048_576).await.unwrap();
    let json_canvas: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    // Pixel [5][5] should be default color (usually #000000), NOT #FF0000
    assert_ne!(json_canvas["pixels"][5][5], "#FF0000"); 

    let _ = fs::remove_dir_all(test_db_path);
}

// Test for GET /updates endpoint
#[tokio::test]
async fn test_updates_endpoint() {
    let test_db_path = "test_db_updates";
    let _ = fs::remove_dir_all(test_db_path);

    let app_state = init_app_state(test_db_path);
    let app = create_router().with_state(app_state);

    // Get time slightly before now (1 sec ago) (simulating a client that just synced 1 sec ago)
    let start_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64 - 1000;

    // Make a pixel update
    let payload = json!({ "x": 10, "y": 10, "color": "#ABCDEF" });
    let _ = app.clone().oneshot(
        Request::builder()
            .uri("/pixel")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap(),
    ).await.unwrap();

    // Poll /updates?since=start_time
    let uri = format!("/updates?since={}", start_time);
    let response = app.oneshot(
        Request::builder()
            .uri(&uri)
            .method("GET")
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = to_bytes(response.into_body(), 1_048_576).await.unwrap();
    let json_body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    // Verify we got the update
    assert_eq!(json_body["updates"].as_array().unwrap().len(), 1);
    assert_eq!(json_body["updates"][0]["color"], "#ABCDEF");
    assert_eq!(json_body["reset_required"], false);

    let _ = fs::remove_dir_all(test_db_path);
}