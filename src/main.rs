extern crate clap;
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

use clap::{App, Arg, ArgMatches, SubCommand};
use prettytable::Table;
use auth0_api::Auth0Api;
use config::Config;
use crate::config::AppConfig;
use crate::user::User;
use ansi_term::Color::{Red, Green};
use std::process::exit;


fn main() {
    let app_arg = || Arg::with_name("app")
        .short("a")
        .takes_value(true)
        .required(true)
        .help("Name of the Auth0 application against which to run queries");

    let list_users_command = SubCommand::with_name("list")
        .arg(app_arg())
        .about("List all users");

    let create_user_command: App = SubCommand::with_name("create")
        .arg(Arg::with_name("email").required(true))
        .arg(Arg::with_name("password").required(true))
        .arg(app_arg())
        .about("Create a user");

    let delete_user_command = {
        let pattern_arg = Arg::with_name("delete-pattern")
            .takes_value(true)
            .help("pattern to find users to delete");

        let id_arg = Arg::with_name("id")
            .long("id")
            .takes_value(true)
            .help("Delete a specific user");

        SubCommand::with_name("delete")
            .arg(pattern_arg)
            .arg(id_arg)
            .arg(app_arg())
            .about("Delete a user by Auth0 ID")
    };

    let config_command = {
        let arg = |long, short| Arg::with_name(long)
            .takes_value(true).long(long).short(short).required(true);

        let display_command = SubCommand::with_name("display")
            .about("Show the current config");

        let validate_command = SubCommand::with_name("validate")
            .about("Validate the current config by trying to parse it");

        let add_command = SubCommand::with_name("add-app")
            .arg(arg("name", "n"))
            .arg(arg("domain", "d"))
            .arg(arg("client-id", "i"))
            .arg(arg("client-secret", "s"))
            .about("Add app details to the config file")
            .alias("add");

        let remove_command = SubCommand::with_name("remove-app")
            .arg(Arg::with_name("app-name").takes_value(true).required(true))
            .about("Remove the config for an Auth0 app from the config.")
            .alias("rm");

        SubCommand::with_name("config")
            .subcommand(display_command)
            .subcommand(validate_command)
            .subcommand(add_command)
            .subcommand(remove_command)
            .about("View, validate or modify the (global) config")
    };

    let completions_command = {
        SubCommand::with_name("completions")
            .about("Generate fish shell completions - pipe std to completions file")
    };

    let app = App::new("auth0")
        .version("0.1")
        .about("Command line app for accessing auth0")
        .subcommand(list_users_command)
        .subcommand(create_user_command)
        .subcommand(delete_user_command)
        .subcommand(config_command)
        .subcommand(completions_command);

    let app_matches = app.clone().get_matches();

    match app_matches.subcommand() {
        ("list", matches) => {
            list_users_main(matches.unwrap());
        }
        ("create", matches) => {
            create_user_main(matches.unwrap());
        }
        ("delete", matches) => {
            delete_user_main(matches.unwrap());
        }
        ("config", matches) => {
            config_main(matches.unwrap());
        }
        ("completions", _) => {
            let name = "auth0-cli";
            let shell = clap::Shell::Fish;
            let output = &mut std::io::stdout();
            app.clone().gen_completions_to(name, shell, output)
        }
        _ => {
            println!("{}", app_matches.usage());
        }
    }
}


fn create_user_main(args: &ArgMatches) {
    let email = args.value_of("email").unwrap();
    let pw = args.value_of("password").unwrap();
    let api = Auth0Api::api_of_commandline_args(args);

    println!("Creating user `{}` with password `{}`", email, pw);
    let resp = api.create_user(email, pw);
    match resp {
        Ok(_) => println!("Successfully created user."),
        Err(err) => println!("Creation failed: {}", err)
    };
}


fn delete_user_main(args: &ArgMatches) {
    let specific_id = args.value_of("id");
    let pattern = args.value_of("delete-pattern");

    if let Some(id) = specific_id {
        let api = Auth0Api::api_of_commandline_args(args);
        match api.delete_user_by_id(id) {
            Ok(_) => println!("Successfully deleted user"),
            Err(err) => println!("Failed to delete user: {}", err)
        };
    } else if let Some(pattern) = pattern {
        let api = Auth0Api::api_of_commandline_args(args);
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
    } else {
        println!("{}", Red.paint("Need to specify pattern or a user id"));
        println!("{}", args.usage());
        exit(1);
    }
}


fn list_users_main(args: &ArgMatches) {
    println!("Listing users");

    let api = Auth0Api::api_of_commandline_args(args);
    let users = api.fetch_users().expect("Failed to fetch users");

    let mut table = Table::new();
    table.add_row(row!["Email", "User ID", "Nickname", "Last Login"]);
    for user in users {
        table.add_row(user.to_table_row());
    }
    table.printstd();
}


fn config_main(args: &ArgMatches) {
    match args.subcommand() {
        ("display", _) => {
            let config_str = config::read_config_file();
            println!("{}", config_str);
        }
        ("validate", _) => {
            let config_str = config::read_config_file();
            match Config::from_string(&config_str) {
                Ok(_) => println!("\n{}\n", Green.paint("Config is valid")),
                Err(err) => println!("\n{}: {}\n", Red.paint("Invalid config"), err)
            };
        }
        ("add-app", matches) => {
            let args = matches.unwrap();
            let config = config::read_config();
            let app_config = AppConfig::of_commandline_args(args)
                .expect("Failed to create app config from command line args.");
            let config = config.add_app(app_config);
            config.persist();
        }
        ("remove-app", matches) => {
            let app_name = matches.and_then(|args| {
                args.value_of("app-name")
            }).unwrap();
            let config = config::read_config().remove_app(app_name);
            config.persist();
        }
        _ => {
            println!("{}", args.usage());
        }
    };
}
