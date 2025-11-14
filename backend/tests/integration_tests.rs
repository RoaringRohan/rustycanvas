// tests/integration_tests.rs

// Integration tests: testing the actual HTTP endpoints

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use backend::server::routes::create_router;
use tower::util::ServiceExt; // for .oneshot()
use serde_json::json;

#[tokio::test]
async fn test_get_endpoint_returns_expected_json() {
    let app = create_router();

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
async fn test_post_endpoint_echoes_json() {
    let app = create_router();

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
