use serde::Serialize;
use std::collections::HashMap;
use toml::de;
use std::path::PathBuf;
use std::fs;
use std::process::exit;


/// Note that I'm not sure if this is portable to windows.
/// Linux defaults to `$HOME/.config`, but MacOS goes to `/Users/$USER/Library/Preferences`
/// where instead I want the same behaviour as linux.
fn config_path() -> PathBuf {
    let home = dirs::home_dir()
        .expect("Could not find home directory to locator config file");
    home
        .join(".config")
        .join("auth0cli")
        .join("config.toml")
}


pub fn read_config_file() -> String {
    let path = config_path();
    let failure_msg = format!("Failed reading config file at: {}", path.display());
    fs::read_to_string(path).expect(&failure_msg)
}


pub fn read_config() -> Config {
    let contents = read_config_file();
    toml::from_str(&contents)
        .expect("Failed to parse config file")
}


#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub name: String,
    pub client_id: String,
    pub client_secret: String,
    pub domain: String,
}


impl AppConfig {
    pub fn new(
        name: String,
        client_id: String,
        client_secret: String,
        domain: String,
    ) -> Result<AppConfig, String> {
        Ok(AppConfig {
            name,
            client_id,
            client_secret,
            domain,
        })
    }
}


#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    apps: HashMap<String, AppConfig>,
    access_tokens: HashMap<String, String>,
}


impl Config {
    pub fn add_app(&self, app: AppConfig) -> Config {
        let mut config = self.clone();
        config.apps.insert(app.name.to_string(), app);
        config
    }

    pub fn remove_app(&self, name: &str) -> Config {
        let mut config = self.clone();
        config.apps.remove(name);
        config
    }

    pub fn from_string(config: &str) -> Result<Config, de::Error> {
        toml::from_str::<Config>(config)
    }

    pub fn get_app_config(&self, app_name: &str) -> Option<&AppConfig> {
        self.apps.get(app_name)
    }

    pub fn persist(&self, verbose: bool) {
        let config_str = toml::to_string(self).unwrap();
        match fs::write(config_path(), &config_str) {
            Ok(()) => {
                if verbose {
                    println!("Persisted new config: \n{}", &config_str)
                } else {
                    println!("Persisted new config.");
                }
            },
            Err(err) => {
                println!("Failed to persist new config: {}", err);
                exit(1);
            }
        };
    }

    pub fn get_access_token(&self, app_name: &str) -> Option<String> {
        self.access_tokens.get(app_name)
            .map(|token| token.clone())
    }

    pub fn add_access_token(&self, app_name: &str, access_token: &str) -> Config {
        let mut config = self.clone();
        config.access_tokens.insert(String::from(app_name), String::from(access_token));
        config
    }
}
