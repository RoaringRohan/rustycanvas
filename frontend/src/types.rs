use serde::{Deserialize, Serialize};

// For GET /canvas
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct CanvasResponse {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Vec<String>>,
}

// For POST /pixel (The Request Body)
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct PixelUpdateInput {
    pub x: u32,
    pub y: u32,
    pub color: String,
}

// For POST /pixel (The Response)
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct PixelUpdateResponse {
    pub success: bool,
    pub error: Option<String>,
}

// For POST /reset (The Response)
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ClearCanvasResponse {
    pub success: bool,
    pub message: String,
}

// Inner Object for Updates (Used inside UpdatesResponse)
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct PixelUpdate {
    pub x: u32,
    pub y: u32,
    pub color: String,
    pub timestamp: u64,
}

// For GET /updates (The Response)
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct UpdatesResponse {
    // Mentioned PixelUpdate struct right above
    pub updates: Vec<PixelUpdate>,
    pub reset_required: bool,
}

// Allowable colours for the palette
pub const PALETTE: &[&str] = &[
    "#000000", // Black
    "#FFFFFF", // White
    "#FF0000", // Red
    "#00FF00", // Green
    "#0000FF", // Blue
    "#FFFF00", // Yellow
    "#00FFFF", // Cyan
    "#FF00FF", // Magenta
];