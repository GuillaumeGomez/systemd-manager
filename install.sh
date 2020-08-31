#!/bin/sh
if [ "$(cat /etc/os-release | grep ubuntu)" ]; then
    sudo apt install libgtk-3-dev
    cargo build --release
    version=$(cat Cargo.toml | grep version | awk -F\" '{print $2}')
    if [ "$(getconf LONG_BIT)" = "64" ]; then arch=amd64; else arch=i386; fi
    mkdir -p debian/usr/bin
    mkdir -p debian/usr/share/applications
    mkdir -p debian/usr/share/polkit-1/actions/
    cp target/release/systemd-manager debian/usr/bin
    cp assets/systemd-manager-pkexec debian/usr/bin/
    cp assets/systemd-manager.desktop debian/usr/share/applications/
    cp assets/org.freedesktop.policykit.systemd-manager.policy debian/usr/share/polkit-1/actions
    dpkg-deb --build debian systemd-manager_${version}_${arch}.deb
    sudo dpkg -i systemd-manager_${version}_${arch}.deb
else
    cargo build --release
    sudo cp target/release/systemd-manager /usr/bin/
    sudo cp assets/systemd-manager-pkexec /usr/bin/
    sudo cp assets/systemd-manager.desktop /usr/share/applications/
    sudo cp assets/org.freedesktop.policykit.systemd-manager.policy /usr/share/polkit-1/actions/
fi
