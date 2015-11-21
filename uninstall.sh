#!/bin/sh
cargo build --release
sudo rm /usr/local/bin/systemd-manager
sudo rm /usr/local/bin/systemd-manager-pkexec
sudo rm /usr/share/applications/systemd-manager.desktop
sudo rm /usr/share/polkit-1/actions/org.freedesktop.policykit.systemd-manager.policy
