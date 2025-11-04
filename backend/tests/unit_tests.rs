// tests/unit_tests.rs

// Unit tests: directly testing handler function logic (not the HTTP endpoints)

use backend::server::handlers::{
    make_test_get_response,
    make_test_post_response,
    TestPostInput,
};
use axum::Json;

#[test]
fn test_make_test_get_response_logic() {
    let response = make_test_get_response();

    assert_eq!(response.status, "ok");
    assert_eq!(response.message, "The test get endpoint handler is working!");
}

#[test]
fn test_make_test_post_response_logic() {
    let input = TestPostInput {
        username: "rohan".to_string(),
        id: 123,
    };

    let response = make_test_post_response(input.clone());

    assert_eq!(response.received, true);
    assert_eq!(response.echo.username, input.username);
    assert_eq!(response.echo.id, input.id);
}