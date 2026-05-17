#!/bin/sh
trap 'echo "got SIGINT"; exit 0' INT
echo "ready"
while true; do sleep 0.05; done
