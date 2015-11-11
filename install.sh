#!/bin/sh
cargo build --release
sudo install target/release/systemd-manager /usr/local/bin/
sudo mv systemd-manager.desktop /usr/share/applications/
