// tests/integration_tests.rs

// Integration tests: testing the actual HTTP endpoints

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use tower::util::ServiceExt; // for .oneshot()
use serde_json::json;
use serial_test::serial;
use backend::server::routes::create_router;
use backend::server::state::init_shared_canvas;

// Test for GET /canvas endpoint
// Verifies that the full canvas is returned correctly
#[tokio::test]
#[serial]
async fn test_canvas_endpoint_returns_full_canvas() {
    // Initialize shared canvas state (loads data/canvas.json)
    let shared_canvas = init_shared_canvas();
    let app = create_router().with_state(shared_canvas);

    // Create a GET request to /canvas
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

    // Verify status code is 200 OK
    assert_eq!(response.status(), StatusCode::OK);

    // Read the body into bytes
    let body_bytes = to_bytes(response.into_body(), 1_048_576)
        .await
        .expect("Failed to read body");

    // Parse bytes into JSON
    let json_body: serde_json::Value =
        serde_json::from_slice(&body_bytes).expect("Body was not valid JSON");

    // Validate top-level fields
    assert_eq!(json_body["width"], 32);
    assert_eq!(json_body["height"], 16);

    // Validate pixel grid dimensions
    let pixels = json_body["pixels"].as_array().expect("pixels is not an array");
    assert_eq!(pixels.len(), 16, "Expected 16 rows of pixels");

    for (i, row) in pixels.iter().enumerate() {
        let row_arr = row.as_array().expect("pixel row is not an array");
        assert_eq!(
            row_arr.len(),
            32,
            "Row {} does not have 32 columns",
            i
        );
    }

    // Optional: Check a known pixel is "#000000"
    assert_eq!(pixels[0][0], "#000000");
}

// Test for POST /pixel endpoint
// Updates a pixel and verifies the update via GET /canvas
#[tokio::test]
#[serial]
async fn test_post_pixel_updates_canvas() {
    let shared_canvas = init_shared_canvas();
    let app = create_router().with_state(shared_canvas);

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

    // GET /canvas to verify
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

    let body_bytes = to_bytes(response_canvas.into_body(), 1_048_576)
        .await
        .unwrap();

    let json_canvas: serde_json::Value =
        serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json_canvas["pixels"][0][0], "#FF0000");
}

// Test for POST /pixel with out-of-bounds coordinates
#[tokio::test]
#[serial]
async fn test_post_pixel_out_of_bounds() {
    let shared_canvas = init_shared_canvas();
    let app = create_router().with_state(shared_canvas);

    let payload = json!({
        "x": 999,
        "y": 999,
        "color": "#123456"
    });

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
}


// ------------------------------------------ TEMPLATE TESTS ------------------------------------------

#[tokio::test]
#[serial]
async fn test_get_endpoint_returns_expected_json() {
    let shared_canvas = init_shared_canvas();
    let app = create_router().with_state(shared_canvas);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/test-get")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = to_bytes(response.into_body(), 1_048_576).await.unwrap();
    let json_body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json_body["status"], "ok");
    assert_eq!(
        json_body["message"],
        "The test get endpoint handler is working!"
    );
}

#[tokio::test]
#[serial]
async fn test_post_endpoint_echoes_json() {
    let shared_canvas = init_shared_canvas();
    let app = create_router().with_state(shared_canvas);

    let payload = json!({
        "username": "rohan",
        "id": 123
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/test-post")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = to_bytes(response.into_body(), 1_048_576).await.unwrap();
    let json_body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json_body["received"], true);
    assert_eq!(json_body["echo"]["username"], "rohan");
    assert_eq!(json_body["echo"]["id"], 123);
}

// ------------------------------------------ TEMPLATE TESTS ------------------------------------------