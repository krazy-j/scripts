#!/bin/bash
# To use this script, simply run it the same way you would run a sleep command, and it will pause Folding@home for the given amount of time.

if [ $# -gt 0 ]; then
	for arg in "$@"; do
		time=$(($time+$(sed 's/d/*24*3600 +/g; s/h/*3600 +/g; s/m/*60 +/g; s/s/\+/g; s/+[ ]*$//g' <<< $arg | bc)))
	done
else
	time=3600
fi

echo -e "\e[31mPausing Folding@home for $(printf "$(($time/3600)):%02d:%02d" $((($time/60)%60)) $(($time%60))).\e[39m"
trap 'echo -e "\nUnpausing Folding@home.";FAHClient -q --send-unpause &> /dev/null' EXIT

FAHClient -q --send-pause &> /dev/null
echo -n "Time remaining: $(printf "$(($time/3600)):%02d:%02d" $((($time/60)%60)) $(($time%60)))"

while [ $time -gt 0 ]; do
	time=$(($time-1))
	echo -n -e "\\rTime remaining: $(printf "$(($time/3600)):%02d:%02d" $((($time/60)%60)) $(($time%60))) "
	sleep 1
done

echo -e "\\r\e[33mDelay has ended.\e[39m       "
echo "Unpausing Folding@home."
FAHClient -q --send-unpause &> /dev/null

trap EXIT
