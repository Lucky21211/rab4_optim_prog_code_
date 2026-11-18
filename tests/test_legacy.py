import csv
from datetime import datetime, timezone
import importlib.util
from pathlib import Path
import random
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('legacy', ROOT / 'services/pascal-legacy/legacy.py')
legacy = importlib.util.module_from_spec(spec)
spec.loader.exec_module(legacy)

class LegacyContractTests(unittest.TestCase):
    def test_names_algorithm_and_ranges(self):
        self.assertTrue(callable(legacy.GetEnvDef))
        a, b = random.Random(42), random.Random(42)
        self.assertEqual(legacy.RandFloat(3.2, 12.6, a), 3.2 + b.random() * (12.6 - 3.2))
        for _ in range(1000):
            self.assertTrue(3.2 <= legacy.RandFloat(3.2, 12.6, a) < 12.6)
            self.assertTrue(-50 <= legacy.RandFloat(-50, 80, a) < 80)

    def test_csv_contract_and_duplicate_protection(self):
        now = datetime(2026, 11, 17, 12, 30, 0, tzinfo=timezone.utc)
        with tempfile.TemporaryDirectory() as folder:
            target = legacy.GenerateAndCopy(folder, now, random.Random(42), import_db=False)
            self.assertEqual(target.name, 'telemetry_20261117_123000.csv')
            with target.open(encoding='utf-8') as stream:
                rows = list(csv.DictReader(stream))
            self.assertEqual(list(rows[0]), legacy.FIELDS)
            self.assertEqual(rows[0]['recorded_at'], '2026-11-17 12:30:00')
            self.assertRegex(rows[0]['voltage'], r'^\d+\.\d{2}$')
            self.assertEqual(rows[0]['source_file'], target.name)
            with self.assertRaises(FileExistsError): legacy.GenerateAndCopy(folder, now, import_db=False)

if __name__ == '__main__': unittest.main(verbosity=2)
