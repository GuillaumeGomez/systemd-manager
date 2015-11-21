#!/bin/sh
cargo build --release
sudo cp target/release/systemd-manager /usr/local/bin/
sudo cp systemd-manager-pkexec /usr/local/bin
sudo chmod +x /usr/local/bin/systemd-manager-pkexec
sudo chmod +x /usr/local/bin/systemd-manager
sudo cp systemd-manager.desktop /usr/share/applications/
sudo cp org.freedesktop.policykit.systemd-manager.policy /usr/share/polkit-1/actions/
