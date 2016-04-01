# Systemd Manager

This application exists to allow the user to manage their systemd services via a GTK3 GUI.

## Screenshot

![](screenshot.png)

## Install Instructions

Simply install Rust via [multirust](https://github.com/brson/multirust) and execute the *install.sh* script. For Ubuntu users, this will automatically install libgtk-3-dev, generate a `systemd-manager` Debian package and automatically install it. For everyone else, it will simply install directly to the /usr prefix.

```sh
./install.sh
```

