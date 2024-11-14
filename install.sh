#!/bin/sh
if grep -q "ubuntu" "/etc/os-release"; then
    sudo apt install libgtk-3-dev
    cargo build --release || exit 1
    version=$(grep "version" "Cargo.toml" | awk -F\" '{print $2}')
    if [ "$(getconf LONG_BIT)" = "64" ]; then arch=amd64; else arch=i386; fi
    mkdir -p debian/usr/bin
    mkdir -p debian/usr/share/applications
    mkdir -p debian/usr/share/polkit-1/actions/
    cp target/release/systemd-manager debian/usr/bin
    cp assets/systemd-manager-pkexec debian/usr/bin/
    cp assets/systemd-manager.desktop debian/usr/share/applications/
    cp assets/org.freedesktop.policykit.systemd-manager.policy debian/usr/share/polkit-1/actions
    sed -i "s/^Version: .*/Version: ${version}/" debian/DEBIAN/control
    dpkg-deb --build debian "systemd-manager_${version}_${arch}.deb" || exit 1
    sudo dpkg -i "systemd-manager_${version}_${arch}.deb" || exit 1
else
    cargo build --release || exit 1
    sudo cp target/release/systemd-manager /usr/bin/
    sudo cp assets/systemd-manager-pkexec /usr/bin/
    sudo cp assets/systemd-manager.desktop /usr/share/applications/
    sudo cp assets/org.freedesktop.policykit.systemd-manager.policy /usr/share/polkit-1/actions/
fi
