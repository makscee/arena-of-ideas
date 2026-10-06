#!/bin/sh
# Samples m1 memory level, load, swap every 2 s while a bench runs; kills the
# bench if the kernel's free level drops under 25% (~4 GB).
BENCH=$1; LOG=$2
while kill -0 $BENCH 2>/dev/null; do
  lvl=$(sysctl -n kern.memorystatus_level)
  echo "$(date +%T) level=$lvl load=$(sysctl -n vm.loadavg | awk '{print $2}') swap=$(sysctl -n vm.swapusage | awk '{print $6}') rss=$(ps -o rss= -p $BENCH)" >> $LOG
  if [ "$lvl" -lt 25 ]; then echo "ABORT level=$lvl" >> $LOG; kill $BENCH; fi
  sleep 2
done
