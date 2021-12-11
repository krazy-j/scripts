#!/bin/bash
update=1
light=false
exit_err() {
	>&2 printf '%s: %s\nTry %s for more information.\n' "$0" "$1" "'$0 --help'"
	exit 2
}

while [[ $1 = -* ]]
do
	case $1 in
		-h|--help)
			echo "Usage: $0 [OPTION]... [NUMBER[SUFFIX]]..."
			echo "Pause for NUMBER seconds. SUFFIX may be 's' for seconds (default), 'm' for minutes, 'h' for hours, or 'd' for days. NUMBER need not be an integer. Given two or more arguments, pause for the amount of time specified by the sum of their values. Given no arguments, pause for one hour."
			echo "  -h, --help           display this help and exit"
			echo "  -m, --minute-update  update the time every minute instead of every second"
			echo "  -q, --quiet          don't output any text"
			echo "  -l, --light          use just one sleep command for delay"
			exit;;
		-m|--minute-update) update=60;;
		-q|--quiet) quiet=true;;
		-l|--light) light=true;;
		*) exit_err "invalid argument '$1'"
	esac
	shift
done
if (($#))
then
	for arg in $@
	do [[ ! $arg =~ ^[0-9]+[smhd]?$ ]] && exit_err "invalid time interval '$arg'"
	done
	((delay=$(sed "s/ /+/g; s/s//g; s/m/*60/g; s/h/*3600/g; s/d/*86400/g" <<< $@)))
else delay=3600
fi

time_left() {
	printf %d:%02d:%02d $(($1/3600)) $(($1%3600/60)) $((update<60?$1%60:0))
}

${quiet-printf '\e[31mPausing Folding@home for %s\e[39m\n' $(time_left $delay)}
trap ${quiet-"printf '\nUnpausing Folding@home\n'; "}'FAHClient -q --send-unpause &> /dev/null' EXIT

FAHClient -q --send-pause &> /dev/null

if $light
then sleep $delay
else
	((end=$(date +%s)+delay))
	while (($(date +%s)<end))
	do
		${quiet-printf '\r\e[KTime remaining - %s' $(time_left $((end-$(date +%s))))}
		sleep $update
	done
fi

${quiet-printf '\r\e[K\e[33mDelay ended\e[39m\nUnpausing Folding@home\n'}
FAHClient -q --send-unpause &>/dev/null

trap EXIT
