// #![allow(unused_imports)]
// #![allow(dead_code)]
#[macro_use]
extern crate prettytable;
extern crate reqwest;
extern crate jsonwebtoken as jwt;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_json;
extern crate crossbeam;
extern crate dirs;

mod user;
mod auth0_api;
mod config;

use structopt::StructOpt;
use prettytable::Table;
use auth0_api::Auth0Api;
use config::Config;
use crate::config::AppConfig;
use crate::user::User;
use ansi_term::Color::{Red, Green};

#[derive(Debug, StructOpt)]
enum ConfigOpts {
    Display,
    Validate,
    #[structopt(about = "Add app to config")]
    Add {
        #[structopt(short, long)]
        app_name: String,
        #[structopt(short = "id", long)]
        client_id: String,
        #[structopt(short, long)]
        domain: String,
        #[structopt(short, long)]
        client_secret: String,
    },
    #[structopt(about = "Remove app from config")]
    Remove {
        #[structopt(short, long)]
        app_name: String
    },
}

#[derive(Debug, StructOpt)]
#[structopt(name = "auth0-cli", about = "CLI to interact with auth0")]
enum Opts {
    #[structopt(name = "create", about = "Create new user")]
    Create {
        email: String,
        password: String,
        #[structopt(short, long)]
        app_name: String,
    },

    #[structopt(name = "list", about = "List users")]
    List {
        #[structopt(short, long)]
        app_name: String,
    },

    #[structopt(name = "delete", about = "Delete user(s)")]
    Delete {
        #[structopt(long)]
        id: Option<String>,
        #[structopt(short, long)]
        pattern: Option<String>,
        #[structopt(short, long)]
        app_name: String,
    },

    #[structopt(about = "Operate on the config file")]
    Config(ConfigOpts),

    #[structopt(about = "Generate completions")]
    Completions, // todo could include different shells.
}

fn main() {
    let opts: Opts = Opts::from_args();
    // println!("OPT: {:?}", opts);

    match opts {
        Opts::Create { email: name, password, app_name } => {
            create_user_main(&name, &password, &app_name);
        }
        Opts::List { app_name } => {
            list_users_main(&app_name);
        }
        Opts::Delete { id, pattern, app_name } => {
            match (id, pattern) {
                (Some(id), None) => delete_user_by_id_main(&id, &app_name),
                (None, Some(pattern)) => delete_users_by_pattern_main(&pattern, &app_name),
                (Some(_), Some(_)) => println!("Can NOT delete users by id & pattern"),
                (None, None) => println!("Can NOT delete users without id or pattern"),
            }
        }
        Opts::Config(ConfigOpts::Display) => config_display_main(),
        Opts::Config(ConfigOpts::Validate) => config_validate_main(),
        Opts::Config(ConfigOpts::Add { app_name, client_id, domain, client_secret }) => {
            config_add_app_main(app_name, client_id, client_secret, domain);
        }
        Opts::Config(ConfigOpts::Remove { app_name }) => {
            config_remove_app_main(&app_name);
        }
        Opts::Completions => {
            let name = "auth0-cli";
            let shell = structopt::clap::Shell::Fish;
            let output = &mut std::io::stdout();
            let mut app: structopt::clap::App = Opts::clap();
            app.gen_completions_to(name, shell, output);
        }
    }
}


fn create_user_main(email: &str, password: &str, app_name: &str) {
    let api = Auth0Api::api_for_app(app_name);

    println!("Creating user `{}` with password `{}`", email, password);
    let resp = api.create_user(email, password);
    match resp {
        Ok(_) => println!("Successfully created user."),
        Err(err) => println!("Creation failed: {}", err)
    };
}

fn delete_user_by_id_main(specific_id: &str, app_name: &str) {
    let api = Auth0Api::api_for_app(app_name);
    match api.delete_user_by_id(specific_id) {
        Ok(_) => println!("Successfully deleted user"),
        Err(err) => println!("Failed to delete user: {}", err)
    };
}

fn delete_users_by_pattern_main(pattern: &str, app_name: &str) {
    let api = Auth0Api::api_for_app(app_name);
    let users = api.fetch_users().expect("Failed to fetch users");
    let matching_users: Vec<User> = users.iter()
        .filter(|user| user.matches(pattern))
        .map(|user| user.clone())
        .collect();

    if matching_users.len() == 0 {
        println!("There were no matching users.");
    } else {
        println!("Going to try & delete {} users.", matching_users.len());
    }

    api.par_delete_users(matching_users);
}


fn list_users_main(app_name: &str) {
    let api = Auth0Api::api_for_app(&app_name);
    let users = api.fetch_users().expect("Failed to fetch users");

    let mut table = Table::new();
    table.add_row(row!["Email", "User ID", "Nickname", "Last Login"]);
    for user in users {
        table.add_row(user.to_table_row());
    }
    table.printstd();
}

fn config_display_main() {
    let config_str = config::read_config_file();
    println!("{}", config_str);
}

fn config_validate_main() {
    let config_str = config::read_config_file();
    match Config::from_string(&config_str) {
        Ok(_) => println!("\n{}\n", Green.paint("Config is valid")),
        Err(err) => println!("\n{}: {}\n", Red.paint("Invalid config"), err)
    };
}

fn config_add_app_main(
    name: String,
    client_id: String,
    client_secret: String,
    domain: String,
) {
    let config = config::read_config();
    let app_config = AppConfig::new(name, client_id, client_secret, domain)
        .expect("Failed to create app config from command line args.");
    let config = config.add_app(app_config);
    config.persist(true);
}

fn config_remove_app_main(app_name: &str) {
    let config = config::read_config().remove_app(app_name);
    config.persist(true);
}
