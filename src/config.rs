use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub payments: PaymentsConfig,
}

#[derive(Debug, Deserialize)]
pub struct PaymentsConfig {
    pub stripe_secret_key: String,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: u32,
}

#[derive(Debug, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub cluster_nodes: Option<Vec<String>>,
}

impl AppConfig {
    pub fn new() -> anyhow::Result<Self> {
        let config = config::Config::builder()
            .add_source(config::File::with_name("config/default"))
            .add_source(config::Environment::with_prefix("FLUXGATE"))
            .build()?;
        
        Ok(config.try_deserialize()?)
    }
}
