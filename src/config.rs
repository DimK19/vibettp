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
}
