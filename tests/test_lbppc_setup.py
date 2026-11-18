"""Account storage tests use fake tokens; no server, credentials or game files."""
import importlib.util
import json
import os
from pathlib import Path
import sys
import tempfile
import types
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('pod_online', ROOT / 'src-tauri/experiments/lbppc_online.py')
online = importlib.util.module_from_spec(spec)
spec.loader.exec_module(online)


class SetupTests(unittest.TestCase):
    def test_settings_merge_and_malformed_file_is_untouched(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'settings.json'
            path.write_text('{"custom":42,"password_protected":"existing"}')
            online.write_settings(path, {'online': True})
            self.assertEqual(json.loads(path.read_text())['password_protected'], 'existing')
            self.assertEqual(json.loads(path.read_text())['custom'], 42)
            path.write_text('broken settings')
            with self.assertRaises(ValueError):
                online.write_settings(path, {'online': False})
            self.assertEqual(path.read_text(), 'broken settings')

    @unittest.skipUnless(os.name == 'nt', 'Windows DPAPI')
    def test_windows_protection_roundtrip(self):
        secret = b'fake-refresh-token-for-test-only'
        locked = online.protect(secret)
        self.assertNotIn(secret, locked)
        self.assertEqual(online.protect(locked, decrypt=True), secret)

    @unittest.skipUnless(os.name == 'nt', 'Windows DPAPI')
    def test_browser_session_is_protected_and_signout_preserves_other_settings(self):
        class Account:
            def __init__(self, load, save): self.save = save; self.username = None
            def signed_in_with(self, tokens):
                self.username = tokens['username']
                self.save({'refresh_token': tokens['refresh_token'], 'username': self.username})
            def sign_out(self): self.save(None)
        fake = types.SimpleNamespace(Account=Account, browser_sign_in=lambda *a, **kw: {
            'access_token': 'fake-access-only', 'refresh_token': 'fake-refresh-only', 'username': 'Test user'})
        with tempfile.TemporaryDirectory() as directory, patch.dict(sys.modules, {'lbppc_signin': fake}):
            root = Path(directory)
            path = root / 'user/launcher.json'
            online.write_settings(path, {'custom': 'keep'})
            self.assertTrue(online.operate(root, 'signin')['signedIn'])
            text = path.read_text()
            self.assertNotIn('fake-refresh-only', text)
            self.assertNotIn('fake-access-only', text)
            self.assertEqual(online.operate(root, 'status')['username'], 'Test user')
            self.assertTrue(online.read_settings(path)['online'])
            online.operate(root, 'signout')
            self.assertFalse(online.operate(root, 'status')['signedIn'])
            self.assertEqual(online.read_settings(path)['custom'], 'keep')

    def test_cancelled_browser_does_not_change_settings(self):
        def cancelled(*args, **kwargs):
            self.assertTrue(kwargs['cancelled']())
            raise RuntimeError('cancelled')
        fake = types.SimpleNamespace(Account=lambda **kw: None, browser_sign_in=cancelled)
        with tempfile.TemporaryDirectory() as directory, patch.dict(sys.modules, {'lbppc_signin': fake}):
            root = Path(directory); marker = root / 'cancel'; marker.touch()
            path = root / 'user/launcher.json'; online.write_settings(path, {'custom': 42})
            before = path.read_bytes()
            with self.assertRaises(RuntimeError): online.operate(root, 'signin', marker)
            self.assertEqual(path.read_bytes(), before)

    def test_file_check_reports_incomplete_and_detected_update(self):
        fake_files = types.SimpleNamespace(check=lambda *a, **kw: types.SimpleNamespace(ok=False, summary=lambda _: 'Missing game files'))
        fake_setup = types.SimpleNamespace(find_update=lambda _: Path('matching-update'))
        with tempfile.TemporaryDirectory() as directory, patch.dict(sys.modules, {'game_files': fake_files, 'game_setup': fake_setup}):
            result = online.operate(Path(directory), 'check', game='base-game')
            self.assertFalse(result['complete'])
            self.assertEqual(result['detectedUpdate'], 'matching-update')
            self.assertEqual(result['message'], 'Missing game files')


if __name__ == '__main__': unittest.main()
