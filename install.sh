#!/usr/bin/env bash

sudo apt-get install libx11-dev

cargo build --release
cp morning.png ~/.local/share/icons
cp target/release/alarm ~/.local/bin/

read -p "domain: " domain
read -p "username: " user
read -s -p "password: " pass

desktop_file="
[Desktop Entry]
Name=Morning
Exec=${HOME}/.local/bin/alarm $domain $user $pass
Type=Application
Icon=${HOME}/.local/share/icons/morning.png
Comment=Set home wakeup alarm
Keywords=Alarm;Morning;Wakeup;
Categories=Utility;"

echo "${desktop_file}" > ~/.local/share/applications/alarm.desktop

echo install done
