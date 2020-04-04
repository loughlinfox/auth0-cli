
use reqwest::Response;
use std::collections::HashMap;
use std::iter::FromIterator;
use crate::user::User;
use crossbeam::unbounded;
use std::thread;
use crate::config::{AppConfig};
use serde_json::{Value};
use std::time;
use std::time::{Duration, SystemTime};
use jwt;
use crate::config;


static N_WORKERS : i32 = 8;


#[derive(Debug, Clone)]
pub struct Auth0Api {
    domain: String,
    access_token: String
}


impl Auth0Api {
    pub fn new(domain: &str, access_token: &str) -> Auth0Api {
        Auth0Api {
            domain: String::from(domain),
            access_token: String::from(access_token)
        }
    }

    pub fn fetch_users(&self) -> Result<Vec<User>, reqwest::Error> {
        let endpoint = format!("https://{}/api/v2/users", self.domain);
        let resp = reqwest::Client::new().get(&endpoint)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send();
        resp?.json()
    }

    pub fn create_user(&self, email: &str, password: &str) -> Result<Response, reqwest::Error> {
        let endpoint = format!("https://{}/api/v2/users", self.domain);
        let data = json!({"email": email, "password": password, "connection": "Username-Password-Authentication"});
        let resp = reqwest::Client::new()
            .post(&endpoint)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .json(&data)
            .send();
        resp
    }

    pub fn delete_user_by_id(&self, user_id: &str) -> Result<Response, reqwest::Error> {
        let endpoint = format!("https://{}/api/v2/users/{}", self.domain, user_id);
        let resp = reqwest::Client::new()
            .delete(&endpoint)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send();
        resp
    }

    fn delete_user(&self, user: &User) {
        println!("Deleting user: (email={}, id={})", user.email, user.user_id);
        let resp = self.delete_user_by_id(&user.user_id);
        match resp {
            Ok(_) => println!("Successfully deleted user: (email={}, id={})", user.email, user.user_id),
            Err(err) => println!("Deletion failed: {}", err)
        };
    }


    pub fn par_delete_users(&self, users: Vec<User>) {
        let (sender, receiver) = unbounded();

        for user in users {
            sender.send(user.clone())
                .expect("Failed sending user across channel to delete");
        }

        let mut workers = vec![];

        for _ in 0..N_WORKERS {
            let receiver = receiver.clone();
            let api = self.clone();
            let handle = thread::spawn(move || {
                while let Ok(user) = receiver.try_recv() {
                    api.delete_user(&user);
                }
            });
            workers.push(handle);
        }

        for worker in workers {
            worker.join().unwrap();
        }
    }
}


impl Auth0Api {
    pub fn api_for_app(app_name: &str) -> Auth0Api {
        let config = config::read_config();

        // let app_name = args.value_of("app").unwrap();
        let failure_msg = format!("Can not find config for app: {}", app_name);
        let app_config = config.get_app_config(app_name).expect(&failure_msg);

        // Cache the token here.
        // Can get away with this for now since the config isn't passed back out of this scope
        // & so old state isn't an issue.
        let access_token = match config.get_access_token(app_name) {
            Some(ref token) if access_token_still_valid(token) => {
                token.to_string()
            },
            _ => {
                let token = fetch_access_token(app_config).expect("Failed to fetch access token");
                config
                    .add_access_token(app_name, &token)
                    .persist();
                token
            }
        };

        Auth0Api::new(&app_config.domain, &access_token)
    }
}


fn fetch_access_token(app_config: &AppConfig) -> Option<String> {
    let endpoint = format!("https://{}/oauth/token", app_config.domain);
    let client = reqwest::Client::new();

    let aud = format!("https://{}/api/v2/", &app_config.domain);
    let data: HashMap<&str, &str> = HashMap::from_iter(vec![
        ("grant_type", "client_credentials"),
        ("client_id", &app_config.client_id),
        ("client_secret", &app_config.client_secret),
        ("audience", &aud),
    ]);

    let resp = client.post(&endpoint)
        .json(&data)
        .header("content-type", "application/json")
        .send();

    let json: Value = resp.expect("Failed to fetch access_token")
        .json().expect("Access token resp not json decode-able");

    match json.get("access_token") {
        Some(Value::String(token)) => Some(token.clone()),
        _ => None
    }
}


fn access_token_still_valid(access_token: &String) -> bool {
    match jwt::dangerous_unsafe_decode::<serde_json::Value>(&access_token) {
        Ok(token) => {
            token.claims.get("exp")
                .and_then(serde_json::Value::as_u64)
                .map(|exp_time_raw| {
                    let exp_time = time::UNIX_EPOCH + Duration::from_secs(exp_time_raw);
                    SystemTime::now() < exp_time
                })
                .unwrap_or(false)
        },
        Err(_err) => {
//            dbg!(err);
            false
        }
    }
}
