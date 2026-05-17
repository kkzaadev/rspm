#!/bin/sh
i=0
while [ $i -lt 50 ]; do
  echo "line $i"
  i=$((i + 1))
done
sleep 5
