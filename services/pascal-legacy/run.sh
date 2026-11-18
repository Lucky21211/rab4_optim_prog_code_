#!/bin/sh
# Compatibility entrypoint; scheduling is owned by cron, not a sleep loop.
exec /entrypoint.sh
