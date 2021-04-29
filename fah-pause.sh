#!/bin/bash
# To use this script, simply run it the same way you would run a sleep command, and it will pause Folding@home for the given amount of time.
# The time defaults to one hour if none is given.
# The process can be stopped at any time (using CTRL+C) to cancel the timer and resume Folding@home.

time=3600

if (($#))
then ((time=$(sed "s/d/*24*3600+/g; s/h/*3600+/g; s/m/*60+/g; s/s/+/g; s/+ *$//g" <<< $@)))
fi

printf "\e[31mPausing Folding@home for %d:%02d:%02d\e[39m\\n" $(($time/3600)) $((($time/60)%60)) $(($time%60))
trap 'echo -e "\nUnpausing Folding@home";FAHClient -q --send-unpause &> /dev/null' EXIT

FAHClient -q --send-pause &> /dev/null
printf "\\rTime remaining - %d:%02d:%02d" $(($time/3600)) $((($time/60)%60)) $(($time%60))

while ((time>0))
do
	((time--))
	printf "\\rTime remaining - %d:%02d:%02d" $(($time/3600)) $((($time/60)%60)) $(($time%60))
	sleep 1
done

echo -e "\\r\e[33mDelay has ended\e[39m        "
echo "Unpausing Folding@home"
FAHClient -q --send-unpause &> /dev/null

trap EXIT
