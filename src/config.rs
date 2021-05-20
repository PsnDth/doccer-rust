use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub id: u64,
    pub token: String,
    pub owner: String,
    pub prefix: String
}
