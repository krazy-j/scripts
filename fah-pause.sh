#!/bin/bash
delay=${1:-1h}
echo -e "\e[31mStopping foldingathome for \e[4m$delay\e[24m.\e[39m"
trap 'echo -e "\n\e[31mStop signal received.\e[39m\nResuming foldingathome.";systemctl start foldingathome' EXIT
systemctl stop foldingathome
sleep $delay
echo -e "\e[33mDelay has ended.\e[39m"
trap 'echo -e "Resuming foldingathome.";systemctl start foldingathome' EXIT
