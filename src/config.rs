// The config file structures for this application

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Habitat {
    pub project: String,
}

#[derive(Debug, Deserialize)]
pub struct FileConfig {
    pub int: Habitat,
    pub stg: Habitat,
    pub prd: Habitat,
}
