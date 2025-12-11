// tests/unit_tests.rs

// Unit tests: directly testing handler function logic (not the HTTP endpoints)

use std::fs;
use backend::server::handlers::{
    make_canvas_response,
    PixelUpdateInput,
    apply_pixel_update,
    reset_canvas_db,
    log_pixel_update,
    fetch_updates_since
};
use backend::server::state::{
    init_app_state,
    CANVAS_WIDTH,
    CANVAS_HEIGHT,
    DEFAULT_COLOR,
    PixelUpdate
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

// Tests for GET /canvas endpoint dependencies
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

// Tests for POST /pixel endpoint dependencies
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

// Tests for POST /reset endpoint dependencies
#[test]
fn test_reset_canvas_logic() {
    let path = "unit_test_reset_logic";
    let db = setup_test_db(path);

    // Paint a pixel manually
    let input = PixelUpdateInput {
        x: 10,
        y: 10,
        color: "#FFFFFF".to_string(),
    };
    apply_pixel_update(&db, &input).unwrap();

    // Verify it's painted
    let response_before = make_canvas_response(&db);
    assert_eq!(response_before.pixels[10][10], "#FFFFFF");

    // Call Reset
    let result = reset_canvas_db(&db);
    assert!(result.is_ok());

    // Verify it's back to default (Black)
    let response_after = make_canvas_response(&db);
    assert_eq!(response_after.pixels[10][10], DEFAULT_COLOR);

    let _ = fs::remove_dir_all(path);
}

// Tests for GET /updates endpoint dependencies
#[test]
fn test_log_pixel_update_adds_to_history() {
    let path = "unit_test_log_update";
    let _ = fs::remove_dir_all(path);
    let app_state = init_app_state(path);

    log_pixel_update(&app_state, 10, 10, "#FFFFFF".to_string());

    let history = app_state.history.read().unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].color, "#FFFFFF");

    let _ = fs::remove_dir_all(path);
}

// Tests for GET /updates endpoint dependencies
#[test]
fn test_history_pruning_limit() {
    let path = "unit_test_pruning";
    let _ = fs::remove_dir_all(path);
    let app_state = init_app_state(path);

    let color = "#000000".to_string();

    // Fill exactly to limit
    for _ in 0..50 {
        log_pixel_update(&app_state, 0, 0, color.clone());
    }

    // Read lock to verify length
    {
        let history = app_state.history.read().unwrap();
        assert_eq!(history.len(), 50);
    }

    // Add one more to trigger prune
    log_pixel_update(&app_state, 99, 99, "#UNIQUE".to_string());

    let history = app_state.history.read().unwrap();
    
    // Length should stay at 50
    assert_eq!(history.len(), 50);
    
    // The LAST item should be our new color
    assert_eq!(history.back().unwrap().color, "#UNIQUE");

    let _ = fs::remove_dir_all(path);
}

// Tests for GET /updates endpoint dependencies
#[test]
fn test_reset_required_logic() {
    let path = "unit_test_reset_req_logic";
    let _ = fs::remove_dir_all(path);
    let app_state = init_app_state(path);

    // Scenario 1: Buffer is NOT full. Client asks for very old time.
    // Should return updates, NO reset.
    {
        let mut history = app_state.history.write().unwrap();
        history.push_back(PixelUpdate { x:0, y:0, color:"#A".to_string(), timestamp: 50 });
    }
    
    let (_, reset) = fetch_updates_since(&app_state, 1000);
    assert_eq!(reset, false, "Should not reset if buffer is not full");

    // Scenario 2: Buffer IS full. Client asks for time older than oldest record.
    // Should trigger RESET.
    {
        let mut history = app_state.history.write().unwrap();
        history.clear();
        // Simulate full buffer [3000, 3001, ... 4999]
        for i in 0..50 {
            history.push_back(PixelUpdate { 
                x:0, y:0, color:"#A".to_string(), 
                timestamp: 3000 + i as u64 
            });
        }
    }

    // Verify manually that length is correct before testing logic
    {
        let history = app_state.history.read().unwrap();
        assert_eq!(history.len(), 50);
        assert_eq!(history.front().unwrap().timestamp, 3000);
    }

    let (_, reset) = fetch_updates_since(&app_state, 1000); // Client asks for T=1000
    // Oldest record is 3000. Buffer is full (50). Client (1000) is older than 3000.
    assert_eq!(reset, true, "Should reset if buffer is full and client is old");

    // Scenario 3: Buffer IS full. Client asks for recent time.
    let (updates, reset) = fetch_updates_since(&app_state, 4990);
    assert_eq!(reset, false, "Should not reset if client is recent");
    assert!(updates.len() > 0);

    let _ = fs::remove_dir_all(path);
}