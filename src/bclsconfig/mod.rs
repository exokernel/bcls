use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Habitat {
    pub project: String,
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct Configurations {
    pub int: Habitat,
    pub stg: Habitat,
    pub prd: Habitat,
}
