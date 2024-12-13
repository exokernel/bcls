//! This module defines the configuration structures used by the application.
//! These structures are used to deserialize configuration data from a TOML file.

use serde::Deserialize;

/// Represents the configuration for a single habitat (environment).
#[derive(Debug, Deserialize)]
pub struct Habitat {
    /// The Google Cloud project ID associated with this habitat.
    pub project: String,
}

/// Represents the overall configuration structure read from the config file.
#[derive(Debug, Deserialize)]
pub struct FileConfig {
    /// Configuration for the integration environment.
    pub int: Habitat,
    /// Configuration for the staging environment.
    pub stg: Habitat,
    /// Configuration for the production environment.
    pub prd: Habitat,
}
