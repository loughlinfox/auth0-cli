#!/usr/bin/env bash


CONFIG_DIR="$HOME/.config/auth0cli"
CONFIG_FILE="$CONFIG_DIR/config.toml"


install_help() {
cat << EOF
This script just ensures that a config directory and file is created.

Config file will be located at:
    ${CONFIG_DIR}

EOF
}


install_main() {
    if [[ ! -d "$CONFIG_DIR" ]]; then
        echo Creating config directory: "$CONFIG_DIR"
        mkdir -p "$CONFIG_DIR"
    fi

    if [[ ! -f "$CONFIG_FILE" ]]; then
        echo Creating empty config.toml at: "$CONFIG_FILE"
        echo "[apps]" > "$CONFIG_FILE"
    fi
}


# Triggers printing help then exiting.
# If no params given then continue to running main function.
while getopts ":h" opt; do
    case "$opt" in
    h )
        install_help
        exit 0
        ;;
    esac
done

install_main
