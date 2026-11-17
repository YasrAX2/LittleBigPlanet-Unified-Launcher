import importlib.util
import io
import os
from pathlib import Path
import struct
import tempfile
import unittest
from unittest.mock import patch

MODULE = Path(__file__).resolve().parents[1] / "src-tauri/experiments/lbppc_native.py"
spec = importlib.util.spec_from_file_location("native_profile", MODULE)
native = importlib.util.module_from_spec(spec)
spec.loader.exec_module(native)


class NativeProfileTests(unittest.TestCase):
    def test_targets_disable_generation_and_keep_30_hz_world(self):
        for fps in (60, 120):
            env = native.environment(fps, Path("test"), "1920x1080", "borderless", "off", {})
            self.assertEqual(env["BB_FRAME_GEN"], "off")
            self.assertEqual(env["BB_FPS_LIMIT"], str(fps))
            self.assertEqual(env["LBPPC_SIM_DIVIDE"], str(fps // 30))
            self.assertEqual(env["BB_ONLINE"], "0")
            self.assertEqual(env["LBPPC_ANIM_STEP_FIX"], "1")

    def test_no_inherited_accounts_research_flags_or_save_paths(self):
        env = native.environment(60, Path("isolated"), "1920x1080", "windowed", "off", {
            "PATH": "preserve", "BB_SHADNET_PASSWORD": "private", "LBPPC_SAVE_DIR": "old",
            "LBPPC_UNCAP_TRIGGER": "old", "BB_WINDOW_MODE": "fullscreen",
            "SHADPS4_HTTP_HOST_OVERRIDES_JSON": "old"})
        self.assertEqual(env["PATH"], "preserve")
        for key in ("BB_SHADNET_PASSWORD", "LBPPC_SAVE_DIR", "LBPPC_UNCAP_TRIGGER",
                    "BB_WINDOW_MODE", "SHADPS4_HTTP_HOST_OVERRIDES_JSON"):
            self.assertNotIn(key, env)
        self.assertEqual(env["LBPPC_DATA_DIR"], "isolated")

    def test_invalid_targets_and_video_options_fail(self):
        for values in ((30, "1920x1080", "windowed", "off"),
                       (120, "invalid", "windowed", "off"),
                       (60, "1920x1080", "invalid", "off")):
            fps, resolution, display, upscale = values
            with self.assertRaises(ValueError):
                native.environment(fps, Path("test"), resolution, display, upscale, {})

    def make_image(self, path):
        size, start = 0x1710000, 128
        with path.open("wb") as f:
            f.write(b"BBPROBE5" + struct.pack("<Q", size))
            f.truncate(size + start)
            for offset, data in {
                0xB2AA: b"\xe8" + struct.pack("<i", 0x254450 - (0xB2AA + 5)),
                0xE7D3CC: struct.pack("<I", 0x3D088889),
                0x3E19D2: struct.pack("<I", 0x3D088889),
                0x3E2A8F: struct.pack("<I", 0x3D088889),
                0x17081FC: bytes.fromhex("c5c858c0c4e3790cc608"),
            }.items():
                f.seek(start + offset)
                f.write(data)

    def test_image_sites_are_read_only_and_wrong_code_is_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "boot.bin"
            self.make_image(path)
            before = path.read_bytes()
            native.check_image(path)
            self.assertEqual(path.read_bytes(), before)
            with path.open("r+b") as f:
                f.seek(128 + 0xB2AA)
                f.write(b"\x90")
            with self.assertRaises(ValueError):
                native.check_image(path)

    def test_invalid_image_and_oversized_image_are_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "bad.bin"
            for data in (b"bad", b"BBPROBE5" + struct.pack("<Q", 2**32) + bytes(40)):
                path.write_bytes(data)
                with self.assertRaises(ValueError):
                    native.check_image(path)

    def test_unrecognized_engine_is_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "LBPPC.exe").write_bytes(b"unknown")
            with self.assertRaises(ValueError):
                native.verify_engine(root)

    def test_hook_failure_stops_child(self):
        from types import SimpleNamespace
        child = SimpleNamespace(stdout=iter([b"LBPPC_SIM_DIVIDE: no memory near the image\n"]),
                                terminate=unittest.mock.Mock())
        with self.assertRaises(RuntimeError):
            next(native.CheckedOutput(child))
        child.terminate.assert_called_once()

    def test_wrapper_restores_environment_after_error(self):
        original = dict(os.environ)
        original_popen = native.subprocess.Popen
        with tempfile.TemporaryDirectory() as tmp, patch.object(native, "verify_engine"), \
                patch.object(native.runpy, "run_path", side_effect=RuntimeError("fixture")), \
                patch("sys.stdout", new=io.StringIO()):
            with self.assertRaisesRegex(RuntimeError, "fixture"):
                native.main([tmp, str(Path(tmp)/"profile"), "60", "game", "", "1920x1080", "windowed", "off"])
        self.assertEqual(dict(os.environ), original)
        self.assertIs(native.subprocess.Popen, original_popen)


if __name__ == "__main__":
    unittest.main()
