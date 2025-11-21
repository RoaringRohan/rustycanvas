// tests/unit_tests.rs

// Unit tests: directly testing handler function logic (not the HTTP endpoints)

use std::fs;
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
    save_canvas_to_file
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
fn test_save_and_load_canvas() {
    let test_filename = "unit_test_save_load.json";

    // Prepare test canvas
    let mut canvas = create_blank_canvas(4, 2);
    canvas.pixels[0][1] = "#FF0000".to_string(); // modify a pixel

    // Save modified version to temp file
    save_canvas_to_file(&canvas, test_filename);

    // Reload from temp file
    let loaded = load_canvas_from_file(test_filename);

    assert_eq!(loaded.width, 4);
    assert_eq!(loaded.height, 2);
    assert_eq!(loaded.pixels[0][1], "#FF0000");

    fs::remove_file(test_filename).expect("Failed to delete test file");
}

#[test]
fn test_canvas_file_exists_after_save() {
    let test_filename = "unit_test_exists.json";

    let canvas = create_blank_canvas(8, 8);
    save_canvas_to_file(&canvas, test_filename);

    assert!(
        fs::metadata(test_filename).is_ok(),
        "Canvas JSON file should exist after saving"
    );

    fs::remove_file(test_filename).expect("Failed to delete test file");
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