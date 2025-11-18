// tests/unit_tests.rs

// Unit tests: directly testing handler function logic (not the HTTP endpoints)

use std::fs;
use serial_test::serial;
use backend::server::handlers::{
    make_test_get_response,
    make_test_post_response,
    TestPostInput,
    PixelUpdateInput,
    apply_pixel_update
};
use backend::server::state::{
    create_blank_canvas,
    load_canvas_from_file,
    save_canvas_to_file,
    CanvasState,
    CANVAS_FILE_PATH,
};

// Tests for GET /canvas endpoint dependencies
#[test]
fn test_create_blank_canvas_dimensions() {
    let canvas = create_blank_canvas(32, 16);

    assert_eq!(canvas.width, 32);
    assert_eq!(canvas.height, 16);
    assert_eq!(canvas.pixels.len(), 16);
    assert_eq!(canvas.pixels[0].len(), 32);
}

#[test]
fn test_create_blank_canvas_default_color() {
    let canvas = create_blank_canvas(4, 3);

    for row in canvas.pixels {
        for pixel in row {
            assert_eq!(pixel, "#000000");
        }
    }
}

#[test]
#[serial]
fn test_save_and_load_canvas() {
    // --- Backup original file ---
    let original = fs::read_to_string(CANVAS_FILE_PATH)
        .expect("Failed to read original canvas.json");
    
    // Prepare test canvas
    let mut canvas = create_blank_canvas(4, 2);
    canvas.pixels[0][1] = "#FF0000".to_string(); // modify a pixel

    // Save modified version
    save_canvas_to_file(&canvas);

    // Reload from file
    let loaded = load_canvas_from_file();

    assert_eq!(loaded.width, 4);
    assert_eq!(loaded.height, 2);
    assert_eq!(loaded.pixels[0][1], "#FF0000");

    // --- Restore original file ---
    fs::write(CANVAS_FILE_PATH, original)
        .expect("Failed to restore original canvas.json");
}

#[test]
#[serial]
fn test_canvas_file_exists_after_save() {
    // --- Backup original file ---
    let original = fs::read_to_string(CANVAS_FILE_PATH)
        .expect("Failed to read original canvas.json");

    let canvas = create_blank_canvas(8, 8);
    save_canvas_to_file(&canvas);

    assert!(
        fs::metadata(CANVAS_FILE_PATH).is_ok(),
        "Canvas JSON file should exist after saving"
    );

    // --- Restore original file ---
    fs::write(CANVAS_FILE_PATH, original)
        .expect("Failed to restore original canvas.json");
}

// Tests for POST /pixel endpoint dependencies
#[test]
fn test_apply_pixel_update_valid() {
    let mut canvas = create_blank_canvas(4, 4);

    let input = PixelUpdateInput {
        x: 1,
        y: 2,
        color: "#FF00FF".to_string(),
    };

    let result = apply_pixel_update(&mut canvas, &input);

    assert!(result.is_ok());
    assert_eq!(canvas.pixels[2][1], "#FF00FF");
}

#[test]
fn test_apply_pixel_update_out_of_bounds() {
    let mut canvas = create_blank_canvas(4, 4);

    let input = PixelUpdateInput {
        x: 10,   // invalid
        y: 0,
        color: "#FFFFFF".to_string(),
    };

    let result = apply_pixel_update(&mut canvas, &input);

    assert!(result.is_err());
}

// ------------------------------------------ TEMPLATE UNIT TESTS ------------------------------------------
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
// ------------------------------------------ TEMPLATE UNIT TESTS ------------------------------------------
