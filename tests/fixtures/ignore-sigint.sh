#!/bin/sh
trap '' INT
echo "ignoring SIGINT"
while true; do sleep 0.05; done
