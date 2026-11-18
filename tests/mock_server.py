"""Local, deterministic upstreams; never uses real third-party credentials."""
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from urllib.parse import urlsplit, parse_qs
from collections import Counter
from threading import Lock
import time

counts = Counter()
modes = {}
lock = Lock()

class Handler(BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        pass

    def reply(self, data, status=200):
        raw = json.dumps(data).encode()
        self.send_response(status)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Content-Length', str(len(raw)))
        if status == 429: self.send_header('Retry-After', '1')
        self.end_headers()
        self.wfile.write(raw)

    def do_POST(self):
        if self.path != '/control': return self.reply({}, 404)
        data = json.loads(self.rfile.read(int(self.headers.get('Content-Length', 0))))
        with lock:
            if data.get('reset'): modes.clear(); counts.clear()
            if 'source' in data: modes[data['source']] = data
        self.reply({'ok': True})

    def do_GET(self):
        parsed = urlsplit(self.path)
        source = parsed.path.strip('/').split('/')[0]
        if source == 'stats': return self.reply(dict(counts))
        with lock:
            counts[source] += 1
            mode = modes.get(source, {})
            remaining = mode.get('failures', -1)
            status = mode.get('status', 200) if remaining != 0 else 200
            if remaining > 0: mode['failures'] -= 1
        if mode.get('delay'): time.sleep(mode['delay'])
        if status != 200: return self.reply({'error': 'fixture failure'}, status)
        if source == 'iss':
            return self.reply({'latitude': 55.75 + counts[source] / 1000, 'longitude': 37.61, 'velocity': 27600, 'altitude': 420, 'timestamp': int(time.time())})
        if source == 'osdr': return self.reply({'OSD-1': {'title': 'Fixture space biology study', 'REST_URL': 'https://visualization.osdr.nasa.gov/biodata/api/v2/dataset/OSD-1/', 'updated_at': '2025-11-01T12:00:00Z'}})
        if source == 'jwst': return self.reply({'body': [{'observation_id': 'fixture-001', 'program': '2734', 'location': os.getenv('FIXTURE_PUBLIC_ORIGIN', 'http://localhost:18080')+'/fixtures/space.jpg', 'details': {'suffix': '_cal', 'instruments': [{'instrument': 'NIRCam'}]}}]})
        if source == 'astro':
            query = parse_qs(parsed.query)
            required = {'from_date', 'to_date', 'time', 'elevation', 'latitude', 'longitude'}
            if not required.issubset(query) or parsed.path.split('/')[-1] not in ('sun', 'moon'): return self.reply({'error': 'invalid upstream contract'}, 400)
            return self.reply({'data': {'rows': [{'body': {'id': parsed.path.split('/')[-1], 'name': parsed.path.split('/')[-1].title()}, 'events': [{'type': 'partial_eclipse', 'eventHighlights': {'peak': {'date': '2026-11-20T12:00:00Z', 'altitude': 15}}}]}]}})
        if source == 'apod': return self.reply({'title': 'Fixture APOD', 'url': 'https://example.org/apod.jpg'})
        if source == 'neo': return self.reply({'element_count': 2, 'near_earth_objects': {}})
        if source in ('flr', 'cme'): return self.reply([{'activityID': 'fixture-001'}])
        if source == 'spacex': return self.reply({'name': 'Fixture launch', 'date_utc': '2026-12-01T00:00:00Z'})
        self.reply({'error': 'not found'}, 404)

if __name__ == '__main__':
    ThreadingHTTPServer(('0.0.0.0', int(os.getenv('PORT', '8080'))), Handler).serve_forever()
