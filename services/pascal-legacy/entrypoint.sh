#!/bin/sh
set -eu
mkdir -p "${CSV_OUT_DIR:-/data/csv}"
schedule="${LEGACY_CRON:-*/5 * * * *}"
# Exactly five cron fields, then our fixed command; no user-supplied command text.
printf '%s\n' "$schedule" | grep -Eq '^([0-9*/,\-]+[[:space:]]+){4}[0-9*/,\-]+$' || { echo 'Invalid LEGACY_CRON' >&2; exit 1; }
printf '%s /usr/local/bin/python /app/legacy.py >>/proc/1/fd/1 2>>/proc/1/fd/2\n' "$schedule" > /etc/crontabs/root
echo '{"event":"legacy_cron_started"}'
exec crond -f -l 2 -L /dev/stdout
