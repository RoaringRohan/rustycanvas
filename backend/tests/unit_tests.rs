// tests/unit_tests.rs

// Unit tests: directly testing handler function logic (not the HTTP endpoints)

use std::fs;
use backend::server::handlers::{
    make_canvas_response,
    PixelUpdateInput,
    apply_pixel_update
};
use backend::server::state::{
    CANVAS_WIDTH,
    CANVAS_HEIGHT,
    DEFAULT_COLOR
};

// Test helper to create a db
fn setup_test_db(path: &str) -> sled::Db {
    let _ = fs::remove_dir_all(path);
    sled::open(path).expect("Failed to open test db")
}

// Tests for GET /canvas endpoint dependencies
#[test]
fn test_default_canvas_values() {
    let path = "unit_test_default_canvas";
    let db = setup_test_db(path);

    let response = make_canvas_response(&db);

    assert_eq!(response.width, CANVAS_WIDTH);
    assert_eq!(response.height, CANVAS_HEIGHT);
    assert_eq!(response.pixels.len(), CANVAS_HEIGHT as usize);
    assert_eq!(response.pixels[0].len(), CANVAS_WIDTH as usize);
    
    // Check that default is black
    assert_eq!(response.pixels[0][0], DEFAULT_COLOR);

    let _ = fs::remove_dir_all(path);
}

#[test]
fn test_persistence_across_restarts() {
    let path = "unit_test_persistence";
    let _ = fs::remove_dir_all(path);

    // Open DB, Write Data, Drop DB
    {
        let db = sled::open(path).unwrap();
        let input = PixelUpdateInput { x: 5, y: 5, color: "#ABCDEF".to_string() };
        apply_pixel_update(&db, &input).unwrap();
        // db is dropped here (simulating server shutdown)
    }

    // Reopen DB (simulating server restart)
    let db_reopened = sled::open(path).unwrap();
    
    // Verify data is still there
    let response = make_canvas_response(&db_reopened);
    assert_eq!(response.pixels[5][5], "#ABCDEF");

    let _ = fs::remove_dir_all(path);
}

// Tests for POST /pixel endpoint dependencies
#[test]
fn test_apply_pixel_update_valid() {
    let path = "unit_test_apply_valid";
    let db = setup_test_db(path);

    let input = PixelUpdateInput {
        x: 1,
        y: 2,
        color: "#FF00FF".to_string(),
    };

    let result = apply_pixel_update(&db, &input);

    assert!(result.is_ok());

    // Verify via response generator
    let response = make_canvas_response(&db);
    assert_eq!(response.pixels[2][1], "#FF00FF");

    let _ = fs::remove_dir_all(path);
}

#[test]
fn test_apply_pixel_update_out_of_bounds() {
    let path = "unit_test_apply_oob";
    let db = setup_test_db(path);

    let input = PixelUpdateInput {
        x: 100, // invalid
        y: 0,
        color: "#FFFFFF".to_string(),
    };

    let result = apply_pixel_update(&db, &input);

    assert!(result.is_err());
    let _ = fs::remove_dir_all(path);
}