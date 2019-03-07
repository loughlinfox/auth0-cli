extern crate reqwest;
#[macro_use]
extern crate serde;
extern crate serde_json;

use std::fs;
use std::collections::HashMap;

type Env = HashMap<String, String>;

fn read_env(path: &str) -> Result<Env, std::io::Error> {
    let env = fs::read_to_string(path)?;
    let mut vars = HashMap::new();
    for line in env.lines() {
        let line = line.trim();
        if line.starts_with("#") {
            continue
        } else {
            let bits: Vec<&str> = line.split("=").collect();
            if bits.len() == 2 {
                let key = bits.first().unwrap().to_string();
                let value = bits.last().unwrap().to_string();
                vars.insert(key, value);
            }
        }
    }
    Ok(vars)
}


#[derive(Debug)]
struct Auth0ManagementApi {
    client_id: String,
    client_secret: String,
    domain: String
}


impl Auth0ManagementApi {
    fn from_env(env: &Env) -> Option<Auth0ManagementApi> {
        let id = env.get("AUTH0_MANAGEMENT_CLIENT_ID").map(String::clone);
        let secret = env.get("AUTH0_MANAGEMENT_CLIENT_SECRET").map(String::clone);
        let domain = env.get("AUTH0_DOMAIN").map(String::clone);
        match (id, secret, domain) {
            (Some(id), Some(secret), Some(domain)) => {
                Some(Auth0ManagementApi { client_id: id, client_secret: secret, domain })
            },
            _ => None
        }
    }

    fn fetch_access_token(self) -> Result<AuthResponse, reqwest::Error> {
        let endpoint = format!("https://{}/oauth/token", self.domain);
        let client = reqwest::Client::new();
        let mut data = HashMap::new();
        data.insert("grant_type", "client_credentials");
        data.insert("client_id", &self.client_id);
        data.insert("client_secret", &self.client_secret);
        let aud = format!("https://{}/api/v2/", self.domain);
        data.insert("audience", &aud);

        let resp = client.post(&endpoint)
            .json(&data)
            .header("content-type", "application/json")
            .send();
        resp?.json()
    }
}


#[derive(Debug, Deserialize)]
struct AuthResponse {
    access_token: String
}


fn main() -> Result<(), std::fmt::Error> {
    let env_path = "./config.env";
    let env = read_env(env_path).unwrap();
    let management_api = Auth0ManagementApi::from_env(&env)
        .expect("Could not find management tokens");
    let resp = management_api.fetch_access_token()
        .expect("Failed to fetch auth token");
    dbg!(resp);

    Ok(())
}