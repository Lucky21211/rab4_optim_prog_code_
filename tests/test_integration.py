"""Acceptance tests run against the isolated compose.test.yml stack."""
import concurrent.futures
from datetime import datetime, timezone
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import time
import unittest
import urllib.request
import urllib.error

WEB = os.getenv('WEB_URL', 'http://localhost:18080')
RUST = os.getenv('RUST_URL', 'http://localhost:18081')
MOCK = os.getenv('MOCK_URL', 'http://mock:8080')

def request(base, path, data=None):
    req = urllib.request.Request(base + path, data=None if data is None else json.dumps(data).encode(), headers={'Content-Type': 'application/json', 'Accept': 'application/json'})
    with urllib.request.urlopen(req, timeout=60) as response:
        raw = response.read().decode()
        return response.status, json.loads(raw)

@unittest.skipUnless(os.getenv('INTEGRATION') == 'true', 'requires isolated Docker Compose test stack')
class AcceptanceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        for _ in range(60):
            try:
                if request(RUST, '/health')[0] == 200: break
            except Exception: time.sleep(1)
        request(MOCK, '/control', {'reset': True})

    def assertError(self, result, code):
        status, data = result
        self.assertEqual(status, 200)
        self.assertFalse(data['ok'])
        self.assertEqual(data['error']['code'], code)
        self.assertRegex(data['error']['trace_id'], r'^[0-9a-f-]{36}$')

    def test_01_collections_and_upsert(self):
        for _ in range(2):
            status, data = request(RUST, '/fetch')
            self.assertEqual(status, 200)
            self.assertIn('id', data)
            self.assertGreater(data['payload']['velocity'], 0)
        _, trend = request(RUST, '/iss/trend?limit=10')
        self.assertTrue(trend['movement'])
        self.assertGreaterEqual(len(trend['points']), 2)
        self.assertTrue({'movement','delta_km','dt_sec','velocity_kmh','from_time','to_time'} <= trend.keys())
        for _ in range(2): self.assertEqual(request(RUST, '/osdr/sync')[1]['written'], 1)
        datasets = request(RUST, '/osdr/list?limit=20')[1]['items']
        self.assertEqual(len([r for r in datasets if r['dataset_id'] == 'OSD-1']), 1)
        self.assertEqual(datasets[0]['title'], 'Fixture space biology study')
        self.assertEqual(len(request(RUST, '/space/refresh')[1]['refreshed']), 5)
        summary = request(RUST, '/space/summary')[1]
        self.assertTrue({'apod','neo','flr','cme','spacex','iss','osdr_count'} <= summary.keys())
        self.assertEqual(summary['apod']['payload']['title'], 'Fixture APOD')

    def test_02_pages_and_web_contracts(self):
        for path in ['/dashboard','/iss','/osdr','/page/welcome','/page/unsafe']:
            with urllib.request.urlopen(WEB + path, timeout=60) as response:
                html = response.read().decode()
                self.assertEqual(response.status, 200)
                self.assertIn('<html', html)
                self.assertNotIn('<script>console.log("XSS training")', html)
                if path == '/dashboard':
                    for label in ['JWST', 'astroForm', 'issSpeedChart', 'CMS']: self.assertIn(label, html)
        self.assertIn('payload', request(WEB, '/api/iss/last')[1])
        self.assertIn('points', request(WEB, '/api/iss/trend')[1])
        gallery = request(WEB, '/api/jwst/feed?source=program&program=2734')[1]
        self.assertEqual(gallery['count'], 1)
        self.assertTrue({'url','obs','program','suffix','inst','caption','link'} <= gallery['items'][0].keys())
        events = request(WEB, '/api/astro/events?lat=55&lon=37&days=7')[1]
        self.assertEqual(len(events['data']), 2)
        self.assertEqual(len(events['events']), 2)

    def test_03_errors_validation_and_retries(self):
        self.assertError(request(RUST, '/osdr/list?limit=100000'), 'VALIDATION_ERROR')
        self.assertError(request(RUST, '/iss/trend?limit=abc'), 'VALIDATION_ERROR')
        self.assertError(request(WEB, '/api/astro/events?lat=999'), 'VALIDATION_ERROR')
        self.assertError(request(WEB, '/api/jwst/feed?program=../etc/passwd'), 'VALIDATION_ERROR')
        request(MOCK, '/control', {'source':'jwst','status':403})
        before = request(MOCK, '/stats')[1].get('jwst', 0)
        self.assertError(request(WEB, '/api/jwst/feed?page=71'), 'UPSTREAM_403')
        self.assertEqual(request(MOCK, '/stats')[1]['jwst'] - before, 1, '403 must not be retried')
        request(MOCK, '/control', {'source':'jwst','status':429,'failures':1})
        self.assertEqual(request(WEB, '/api/jwst/feed?page=72')[1]['count'], 1)
        request(MOCK, '/control', {'source':'iss','status':200})

    def test_04_cache_prevents_duplicate_external_calls(self):
        request(MOCK, '/control', {'source':'jwst','status':200})
        before = request(MOCK, '/stats')[1].get('jwst', 0)
        with concurrent.futures.ThreadPoolExecutor(max_workers=6) as executor:
            results = list(executor.map(lambda _: request(WEB, '/api/jwst/feed?page=81'), range(10)))
        self.assertTrue(all(data.get('count') == 1 for _, data in results))
        self.assertEqual(request(MOCK, '/stats')[1]['jwst'] - before, 1)

    def test_05_job_lock_prevents_overlap(self):
        request(MOCK, '/control', {'source':'iss','status':200,'delay':0.5})
        before = request(MOCK, '/stats')[1].get('iss', 0)
        with concurrent.futures.ThreadPoolExecutor(max_workers=6) as executor:
            results = list(executor.map(lambda _: request(RUST, '/fetch'), range(6)))
        good = sum('id' in data for _, data in results)
        busy = sum(data.get('error', {}).get('code') == 'JOB_BUSY' for _, data in results)
        self.assertGreaterEqual(good, 1)
        self.assertGreater(busy, 0)
        self.assertEqual(request(MOCK, '/stats')[1]['iss'] - before, good)
        request(MOCK, '/control', {'source':'iss','status':200})

    def test_06_database_types_bounded_cache_and_legacy(self):
        import psycopg
        with psycopg.connect() as connection:
            with connection.cursor() as cur:
                cur.execute("SELECT fetched_at FROM iss_fetch_log LIMIT 1")
                self.assertIsNotNone(cur.fetchone()[0].tzinfo)
                cur.execute("SELECT count(*) FROM space_cache")
                self.assertEqual(cur.fetchone()[0], 5)
        root = Path(__file__).resolve().parents[1]
        spec = importlib.util.spec_from_file_location('legacy_acceptance', root / 'services/pascal-legacy/legacy.py')
        legacy = importlib.util.module_from_spec(spec); spec.loader.exec_module(legacy)
        with tempfile.TemporaryDirectory() as folder:
            target = legacy.GenerateAndCopy(folder, datetime(2026,1,1,tzinfo=timezone.utc), import_db=False)
            legacy.ImportCSV(target); legacy.ImportCSV(target)
            with psycopg.connect() as connection:
                count = connection.execute('SELECT count(*) FROM telemetry_legacy WHERE source_file=%s', (target.name,)).fetchone()[0]
                self.assertEqual(count, 1)

    def test_07_csrf_upload_is_not_exempted(self):
        req = urllib.request.Request(WEB + '/upload', data=b'', method='POST')
        with self.assertRaises(urllib.error.HTTPError) as failure: urllib.request.urlopen(req)
        self.assertEqual(failure.exception.code, 419)

if __name__ == '__main__': unittest.main(verbosity=2)
