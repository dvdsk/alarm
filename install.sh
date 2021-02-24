#!/usr/bin/env bash

cp alarm.desktop ~/.local/share/applications/
cp morning.png ~/bin/
cp target/release/alarm ~/bin/

desktop_file="
[Desktop Entry]
Name=Morning
Exec=${HOME}/bin/start_alarm.sh
Path=${HOME}/bin/
Type=Application
Icon=${HOME}/bin/morning.png
Comment=Set home wakeup alarm
Keywords=Alarm;Morning;Wakeup;
Categories=Utility;"

echo "${desktop_file}" > ~/.local/share/applications/alarm.desktop

echo "./alarm https://domain:port user passw" > ~/bin/start_alarm.sh 
echo please edit the example start_alarm.sh file in ~/bin/
