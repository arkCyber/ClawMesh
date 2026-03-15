/// ClawMesh Database Schema Module
/// 
/// This module provides full integration with Lemmy's mature database schema
/// ensuring 100% compatibility and leveraging all of Lemmy's proven data structures

pub mod lemmy_schema_integration;

// Re-export all Lemmy schema components for easy access
pub use lemmy_db_schema::*;
pub use lemmy_db_schema_file::*;

// Re-export our integration functions
pub use lemmy_schema_integration::*;
