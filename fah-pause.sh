#!/bin/bash
update=1
quiet=false
light=false

while (($#))
do
	if [[ $1 = -* ]]
	then
		case $1 in
			-h|--help)
				echo "Usage: $0 [OPTION]... [NUMBER[SUFFIX]]..."
				echo "Pause for NUMBER seconds. SUFFIX may be 's' for seconds (default), 'm' for minutes, 'h' for hours, or 'd' for days. NUMBER need not be an integer. Given two or more arguments, pause for the amount of time specified by the sum of their values. Given no arguments, pause for one hour."
				echo "  -h, --help           display this help and exit"
				echo "  -m, --minute-update  update the time left every minute instead of every second"
				echo "  -q, --quiet          don't output any text"
				echo "  -l, --light          don't output any text"
				exit;;
			-m|--minute-update) update=60;;
			-q|--quiet) quiet=true;;
			-q|--quiet) light=true;;
			*)
				echo "$0: unrecognized argument '$1'"
				echo "Try '$0 --help' for more information."
				exit 1
		esac
	elif [[ $1 =~ ^[0-9]+[smhd]?$ ]]
	then ((delay+=$(sed "s/s//; s/m/*60/; s/h/*3600/; s/d/*86400/" <<< $1)))
	else
		echo "$0: invalid time interval '$1'"
		echo "Try '$0 --help' for more information."
		exit 1
	fi
	shift
done

((delay)) || delay=3600
timeleft() { echo $(($1/3600))$(printf :%02d $((($1/60)%60)))$( ((update==60)) || printf :%02d $(($1%60))); }

$quiet || printf "\e[31mPausing Folding@home for $(timeleft $delay)\e[39m\n"
trap '$quiet && echo -e "\nUnpausing Folding@home"; FAHClient -q --send-unpause &> /dev/null' EXIT

FAHClient -q --send-pause &> /dev/null

if $light
then sleep $delay
else
	$quiet || printf "Time remaining - $(timeleft $delay)"
	((end=$(date +%s)+delay))
	while (($(date +%s)<end))
	do
		sleep $update
		$quiet || printf "\033[1K\rTime remaining - $(timeleft $((end-$(date +%s))))"
	done
fi

$quiet || echo -e "\033[1K\r\e[33mDelay ended\e[39m\nUnpausing Folding@home"
FAHClient -q --send-unpause &> /dev/null

trap EXIT
