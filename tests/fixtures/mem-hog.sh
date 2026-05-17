#!/bin/sh
# Allocate a large environment variable so RSS visibly grows then sleep.
RSPM_BIG=$(printf '%*s' 200000 '' | tr ' ' 'x')
export RSPM_BIG
echo "allocated"
sleep 30
