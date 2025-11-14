// server/mod.rs

// This file defines the 'server' module and exposes the submodules to the rest of the project

// For your knowledge
// mod.rs is necessary if you want to make a module (folder) with rust files in it
// We would declare the files here to make them part of the 'server' module (notice how the folder name is 'server')

// Now if we want to use these modules below in other parts of the project, we just have to declare it at the top of the file
// E.g. 'use crate::server::routes;' for routes.rs

pub mod routes;
pub mod handlers;
pub mod state;