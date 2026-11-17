"""Opt-in, offline rendering research using unmodified LBPPC 0.1.5 hooks.

No game files are modified. The 30 Hz world update is retained; 60/120 Hz
simulation and interpolated world motion are not implemented by this profile.
"""
import hashlib
import os
from pathlib import Path
import runpy
import struct
import subprocess
import sys

HASHES = {
    "LBPPC.exe": "e3f2da1b3a4799fbec95784271c37a1aa2b1949845298d4ccf27795668c14d93",
    "run.py": "441c699f3e1b05ca155d3ab88206f008fcf76083500fc421bf44fffb2cb58671",
    "bin/LBPPC-Game.exe": "dfa36d62c1d440c03f81581e41af61bce3360c0bc75fd5a434d98fa562d2a8d1",
}


def verify_engine(root):
    for name, expected in HASHES.items():
        path = root / name
        if path.stat().st_size > 64 * 1024 * 1024:
            raise ValueError("Engine file exceeds the supported size.")
        if hashlib.sha256(path.read_bytes()).hexdigest() != expected:
            raise ValueError("Native tests require unmodified official LBPPC 0.1.5.")


def check_image(path):
    """Check research sites in the prepared image without exporting game bytes."""
    length = path.stat().st_size
    with path.open("rb") as image:
        header = image.read(56)
        if len(header) != 56 or header[:8] != b"BBPROBE5":
            raise ValueError("Unsupported prepared game format; native test stopped.")
        size = struct.unpack_from("<Q", header, 8)[0]
        if not 0x1708206 < size <= 512 * 1024 * 1024 or length < size + 56:
            raise ValueError("Invalid prepared game image size.")
        start = length - size
        checks = {
            0xB2AA: b"\xe8" + struct.pack("<i", 0x254450 - (0xB2AA + 5)),
            0xE7D3CC: struct.pack("<I", 0x3D088889),
            0x3E19D2: struct.pack("<I", 0x3D088889),
            0x3E2A8F: struct.pack("<I", 0x3D088889),
            0x17081FC: bytes.fromhex("c5c858c0c4e3790cc608"),
        }
        for offset, expected in checks.items():
            image.seek(start + offset)
            if image.read(len(expected)) != expected:
                raise ValueError("Game research sites differ; native test stopped before boot.")


def environment(fps, profile, resolution, display, upscaling, inherited):
    if fps not in (60, 120):
        raise ValueError("Native test target must be 60 or 120.")
    if resolution not in ("1280x720", "1920x1080", "2560x1440", "3840x2160",
                          "2560x1080", "3440x1440", "3840x1600", "3840x1080", "4480x1260"):
        raise ValueError("Unsupported resolution.")
    if display not in ("windowed", "borderless", "fullscreen") or upscaling not in ("off", "fsr1"):
        raise ValueError("Unsupported video option.")
    # Credentials, online switches and previous research flags must not enter a test.
    env = {k: v for k, v in inherited.items()
           if not k.upper().startswith(("BB_", "LBPPC_", "SHADPS4_"))}
    env.update({
        "LBPPC_DATA_DIR": str(profile), "BB_CONFIG": str(profile / "native.ini"),
        "LBPPC_SIM_DIVIDE": str(fps // 30), "LBPPC_ANIM_STEP_FIX": "1",
        "LBPPC_SORT_KEYS_FIX": "1", "LBPPC_SHAPE_FIX": "1", "LBPPC_SPU_NAN_GUARD": "1",
        "LBPPC_FLIP_RATE": "0", "BB_FPS_LIMIT": str(fps), "BB_VBLANK_HZ": str(fps),
        "BB_LOW_LATENCY": "off", "BB_FRAME_GEN": "off", "BB_ONLINE": "0",
        "BB_FRAME_STATS": "1", "BB_FPS_LOG": "1", "LBPPC_RENDER_RES": resolution,
        "BB_FSR1": "1" if upscaling == "fsr1" else "0",
        # The original loop's vsync interval is zeroed in memory; a late write
        # reapplies it after initialization. These are fixed, checked-build sites.
        "LBPPC_POKE": "1060a10:00000000", "LBPPC_POKE_LATE": "1060a10:00000000",
        "LBPPC_POKE_AT": "1",
    })
    if display != "windowed":
        env["BB_WINDOW_MODE"] = display
    return env


class CheckedOutput:
    def __init__(self, process):
        self.process = process
        self.stream = process.stdout

    def __iter__(self):
        return self

    def __next__(self):
        line = next(self.stream)
        research = (b"LBPPC_SIM_DIVIDE:", b"Animation step:", b"SPU NaN guard:")
        if any(prefix in line for prefix in research) and (
                b"not applied" in line or b"no memory" in line):
            self.process.terminate()
            raise RuntimeError("Native hook failed; game stopped to avoid incorrect timing.")
        return line


def main(argv):
    if len(argv) != 8:
        raise ValueError("Expected engine, test profile, FPS, game, update and video options.")
    root, profile = (Path(value).resolve() for value in argv[:2])
    fps = int(argv[2])
    verify_engine(root)
    env = environment(fps, profile, *argv[5:], os.environ)
    profile.mkdir(parents=True, exist_ok=True)
    (profile / "native.ini").write_text("frame_gen=off\nshow_fps=1\nshow_frame_times=1\ngraph_game_frames=1\n", encoding="utf-8")
    # Frozen GUI Python can start without console streams.
    if sys.stdout is None:
        sys.stdout = (profile / "launcher.log").open("w", encoding="utf-8", buffering=1)
    if sys.stderr is None:
        sys.stderr = sys.stdout
    print(f"Patchwork native rendering test: target {fps}, frame generation OFF, physics 30 Hz.", flush=True)
    print("Experimental: wrong speed, stepped motion and rendering faults remain possible. Test saves are isolated.", flush=True)
    old_popen, old_env, old_argv = subprocess.Popen, dict(os.environ), sys.argv
    children = []
    game_exe = (root / "bin/LBPPC-Game.exe").resolve()

    def popen(command, *args, **kwargs):
        is_game = isinstance(command, (list, tuple)) and Path(command[0]).resolve() == game_exe
        if is_game:
            check_image(Path(command[1]))
            # run.py derives some graphics flags again. Explicit test flags win
            # only for the game child; its normal runtime defaults are retained.
            child_env = dict(kwargs.get("env", os.environ))
            child_env.update({key: value for key, value in env.items()
                              if key.startswith(("BB_", "LBPPC_"))})
            kwargs["env"] = child_env
        child = old_popen(command, *args, **kwargs)
        if is_game:
            children.append(child)
            if child.stdout is not None:
                child.stdout = CheckedOutput(child)
        return child

    try:
        os.environ.clear()
        os.environ.update(env)
        subprocess.Popen = popen
        sys.path.insert(0, str(root))
        sys.argv = [str(root / "run.py"), argv[3]]
        if argv[4]:
            sys.argv.extend(["--update", argv[4]])
        runpy.run_path(str(root / "run.py"), run_name="__main__")
    finally:
        subprocess.Popen = old_popen
        sys.argv = old_argv
        os.environ.clear()
        os.environ.update(old_env)
        for child in children:
            if child.poll() is None:
                child.terminate()
                child.wait(timeout=10)


if __name__ == "__main__":
    try:
        main(sys.argv[1:])
    except (ValueError, OSError, RuntimeError) as error:
        print(f"Native test stopped: {error}", file=sys.stderr)
        sys.exit(1)
