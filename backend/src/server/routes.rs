// server/routes.rs

// This file defines the routes for the Axum web server

// For your knowledge
// A route maps the HTTP request and a URL path to a specific handler function

use axum::{Router, routing::{get, post}};
use crate::server::state::AppState;
use crate::server::handlers::{
    get_canvas_handler,
    update_pixel_handler,
    reset_canvas_handler,
};

// Function to create and return the router with all defined routes
pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/canvas", get(get_canvas_handler))
        .route("/pixel", post(update_pixel_handler))
        .route("/reset", post(reset_canvas_handler))
}