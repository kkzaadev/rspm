#!/bin/sh
sleep 0.05
if [ -n "${RSPM_READY_FILE}" ]; then
  touch "$RSPM_READY_FILE"
fi
sleep 5
