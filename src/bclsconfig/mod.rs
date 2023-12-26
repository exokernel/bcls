// The config file structures for this application

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Habitat {
    pub project: String,
    pub token: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Configurations {
    pub int: Habitat,
    pub stg: Habitat,
    pub prd: Habitat,
}
