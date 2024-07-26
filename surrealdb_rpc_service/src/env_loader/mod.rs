use serde_json::Value;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

const ENV_PATH: &str = "dependencies/env.json";

/// Load the environment variables from the env.json file
pub fn load() -> Result<(), Box<dyn Error>> {
    // Open the env.json file
    let file = File::open(ENV_PATH)?;
    let file = BufReader::new(file);
    let env_map: Value = serde_json::from_reader(file)?;

    // Set the environment variables
    let variables = match env_map["variables"].as_array() {
        Some(variables) => variables,
        None => return Err("Invalid env.json format".into()),
    };
    for v in variables {
        let key = match v["key"].as_str() {
            Some(key) => key,
            None => continue,
        };
        let value = match v["value"].as_str() {
            Some(value) => value,
            None => continue,
        };
        env::set_var(key, value);
    }

    Ok(())
}
