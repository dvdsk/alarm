#!/usr/bin/bash
set -e

if [ -z $3 ]; then
	echo need 3 arguments: url user passw
	exit -1
fi

bash ./crosscomp.sh --release

scp target/aarch64-unknown-linux-gnu/release/alarm phone:/home/manjaro/.local/bin/alarm
scp morning.png phone:/home/manjaro/.local/share/icons/morning.png

cp alarm.desktop /tmp/alarm.desktop
sed -i "s@<URL>@$1@g" /tmp/alarm.desktop
sed -i "s/<USER>/$2/g" /tmp/alarm.desktop
sed -i "s/<PASSW>/$3/g" /tmp/alarm.desktop
scp /tmp/alarm.desktop phone:/home/manjaro/.local/share/applications/alarm.desktop
rm /tmp/alarm.desktop
