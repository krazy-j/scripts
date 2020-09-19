#!/bin/bash
archive=$1
archivefile=$2
updatedelay=${3:-1s}
if test -f "$archive"; then
	echo -e "Beginning live extract. Updating every \e[4m$updatedelay\e[24m."
	trap 'echo -e "\n\e[31mStop signal received.\e[39m\nHalting live copy and cleaning up.";rm $archivefile' EXIT
	unzip -j "$archive" "$archivefile"
	while true; do
		sleep $updatedelay
		unzip -j -q -o "$archive" "$archivefile"
		echo -n -e "\\rUpdated at $(date "+%T")"
	done
else
	echo -e "\n\e[31mERROR: $archive not found!\e[39m"
fi
