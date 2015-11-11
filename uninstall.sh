#!/bin/sh
cargo build --release
sudo rm /usr/local/bin/systemd-manager
sudo rm /usr/share/applications/systemd-manager.desktop
