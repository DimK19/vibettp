use serde::Deserialize;

/*
#[derive(Deserialize)] is a Rust attribute macro that tells the compiler to automatically
generate code to allow a struct to be deserialized — in this case, from a format like TOML,
JSON, YAML, etc. Used to load structured data (like TOML) into Rust structs.
*/
#[derive(Deserialize)]
pub struct Config {
    pub root_directory: String,
    pub keep_alive: bool,
    pub timeout_seconds: u64,
    pub max_clients: usize,
    pub bind_address: String,
    pub port: u16,
}

#[cfg(test)]
mod tests {
    use std::fs;
    use super::*;

    #[test]
    fn test_config_defaults() {
        let raw = fs::read_to_string("config.toml").expect("❌ Failed to read config file");
        let config: Config = toml::from_str(&raw).expect("❌ Failed to parse config");
        assert_eq!(config.bind_address, "127.0.0.1");
        assert_eq!(config.port, 7878);
    }
}
