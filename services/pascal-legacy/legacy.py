"""Contract-compatible replacement of Pascal LegacyCSV; one execution per cron tick."""
import argparse
import csv
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import random
import sys

FIELDS = ['recorded_at', 'voltage', 'temp', 'source_file']

def GetEnvDef(name, default):
    return os.environ.get(name) or default

def RandFloat(minV, maxV, rng=None):
    return minV + (rng or random).random() * (maxV - minV)

def GenerateAndCopy(out_dir=None, recorded_at=None, rng=None, import_db=True):
    now = recorded_at or datetime.now(timezone.utc)
    directory = Path(out_dir or GetEnvDef('CSV_OUT_DIR', '/data/csv'))
    directory.mkdir(parents=True, exist_ok=True)
    filename = 'telemetry_' + now.strftime('%Y%m%d_%H%M%S') + '.csv'
    target = directory / filename
    # Exclusive creation prevents overlapping ticks from overwriting a pending CSV.
    with target.open('x', newline='', encoding='utf-8') as stream:
        writer = csv.writer(stream, lineterminator='\n')
        writer.writerow(FIELDS)
        writer.writerow([now.strftime('%Y-%m-%d %H:%M:%S'), f'{RandFloat(3.2, 12.6, rng):.2f}', f'{RandFloat(-50.0, 80.0, rng):.2f}', filename])
    if import_db:
        ImportCSV(target)
    return target

def ImportCSV(target):
    import psycopg
    # libpq reads PGHOST/PGPORT/PGUSER/PGPASSWORD/PGDATABASE from the container.
    with psycopg.connect(options='-c timezone=UTC', connect_timeout=5) as connection:
        with connection.cursor() as cursor:
            cursor.execute('SELECT pg_advisory_xact_lock(%s)', (74204001,))
            cursor.execute('CREATE TEMP TABLE incoming (LIKE telemetry_legacy INCLUDING DEFAULTS) ON COMMIT DROP')
            with cursor.copy('COPY incoming(recorded_at,voltage,temp,source_file) FROM STDIN WITH (FORMAT csv, HEADER true)') as copy:
                copy.write(Path(target).read_bytes())
            cursor.execute('''INSERT INTO telemetry_legacy(recorded_at,voltage,temp,source_file)
                SELECT recorded_at,voltage,temp,source_file FROM incoming
                ON CONFLICT(source_file,recorded_at) DO NOTHING''')
            count = cursor.rowcount
    print(json.dumps({'event': 'legacy_imported', 'file': Path(target).name, 'rows': count}), flush=True)

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--csv-only', action='store_true')
    parser.add_argument('--import-file')
    args = parser.parse_args()
    try:
        if args.import_file:
            ImportCSV(args.import_file)
        else:
            path = GenerateAndCopy(import_db=not args.csv_only)
            print(json.dumps({'event': 'legacy_generated', 'file': path.name}), flush=True)
    except Exception as error:
        # No DSN, password, SQL text or payload is copied into logs.
        print(json.dumps({'event': 'legacy_failed', 'type': type(error).__name__}), file=sys.stderr, flush=True)
        return 1
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
