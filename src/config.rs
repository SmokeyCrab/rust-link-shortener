use serde::{ Deserialize, Serialize };
use std::fs;
use std::error::Error;

#[derive(Deserialize)]
pub struct PostgresConfig {
    pub postgres_ip: String,
    pub postgres_port: String,
    pub postgres_user: String,
    pub postgres_database_name: String,
    pub postgres_password: String,
    pub postgres_table: String,
}

#[derive(Deserialize)]
pub struct HostConfig {
    pub host_ip: String,
    pub host_port: String,
}

// pub enum ConfigError {
//     serde_json::Error,
//     std::io::Error

// }

pub fn get_config() -> Result<(PostgresConfig, HostConfig), Box<dyn Error + Send + Sync>> {
    let config_json: String = fs::read_to_string("config.json")?;
    let pg_config: PostgresConfig = serde_json::from_str(&config_json[..])?;
    let host_config: HostConfig = serde_json::from_str(&config_json[..])?;

    Ok((pg_config, host_config))
}
