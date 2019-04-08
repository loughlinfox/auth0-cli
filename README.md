
### What
Simple command line util for creating, deleting & listing users in an Auth0 app.


### Why
Why the tool?
b/c opening the auth0 ui and poking around the users for test/staging deployments is tedious.

Why rust? 
b/c I'm trying to learn it and decided something I might use would be a good way of doing so.
Probably/definitely overkill.


### Install & use

1. Get rust https://www.rust-lang.org/tools/install.
2. `cd` into project
3. run installer script to ensure config directories exist `./install.sh`
4. build project with `cargo build --release`
5. install (locally) with `cargo install --path .`
6. use it `auth0-cli --help`
