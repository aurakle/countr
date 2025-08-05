use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub postgres_db: String,
    #[serde(default = "default_backend_port")]
    pub port: u16,
}

pub fn get_config() -> &'static Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();
    CONFIG.get_or_init(|| {
        println!("Reading environment variables...");
        serde_env::from_env().expect("Failed to parse environment variables")
    })
}

fn default_backend_port() -> u16 {
    8080
}
