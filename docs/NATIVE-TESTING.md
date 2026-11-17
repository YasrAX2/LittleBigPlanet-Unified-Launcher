# Experimental native rendering tests

Version 1.4.0 is a prerelease for user testing. The stable launcher remains available separately.

## Start a test

1. Install or import the unmodified official **LBPPC 0.1.5 Windows release**. The launcher checks exact SHA-256 fingerprints of LBPPC.exe, run.py and bin/LBPPC-Game.exe. Other versions are refused in this experimental mode.
2. Import your own LBP3 US CUSA00473 or European CUSA00063 game and matching **1.28** update. Original game files are not modified or distributed.
3. Open LBP3, enable video overrides, choose **Native rendering - experimental test**, then select **60** or **120 FPS**. Set a resolution your hardware can sustain.
4. Press Play. These profiles bypass the engine's graphical launcher using its official --script interface, keep frame generation off and run offline. First preparation can take time.
5. Use **Open test logs and saves** to inspect the result. The profiles are under the launcher's local data folder, native-tests/60 or native-tests/120. No files are uploaded automatically.

## What this changes

These are requests for independently rendered frames using the existing upstream LBPPC_SIM_DIVIDE, animation/NaN guards, flip-rate and frame-limit research hooks. No custom engine binary is shipped. Upstream does not advertise these as stable native-FPS support.

The world simulation remains **30 Hz**, with one world update per two/four render-loop iterations. This does not implement 60/120 Hz game simulation or world-transform interpolation. Character motion can still step at 30 Hz. If the renderer cannot hold its target, game speed can be wrong. Initialization, materials, animation and SPU work can still fail. A 120 FPS selection is not proof that 120 unique frames reached your display.

The original loop's vsync interval is changed only in process memory through the port's fixed research hook. The initial value is reapplied after initialization. This has not been gameplay-tested here. Changing it is not a complete timing fix for every subsystem.

Engine fingerprints and expected prepared-game call/animation/SPU code sites are checked before boot. A reported failure to install the simulation/animation guard stops the child instead of continuing with a known timing failure. These checks reduce avoidable mistakes; they do not establish correctness in play.

## Isolation and returning to normal

Test saves, generated game cache, graphics settings and logs are separate from the regular engine's user/out folders. Existing accounts and online settings are not loaded by the test profile. Normal launcher/engine settings are not overwritten by a native test.

Choose **Standard** rendering, or disable video overrides, to return to the normal engine launch. Before downgrading to version 1.3.0, choose Standard and save; the older version cannot read the experimental rendering field. Separately exported configuration backups remain private.

## What to report after testing

Test 60 first, then 120. Record whether the game reaches the Pod, whether its FPS counter holds the target, and whether walking, jumping, falling, camera motion, animation and sound keep normal speed compared with Standard 30 FPS. Test a simple level and a busy scene. Describe freezes, missing shapes or doubled/halved speed. The local out/last_run.log contains the engine's frame statistics. Review logs before sharing; no automatic report upload exists.

No actual native-FPS, gameplay-speed or final desktop visual result is claimed by the automated tests.

## Source and verification

- [Official LBPPC source](https://github.com/emurphy04/LBPPC/blob/main/src/probe.c#L1211-L1335)
- [Official 0.1.5 release](https://github.com/emurphy04/LBPPC/releases/tag/v0.1.5)
- Verified official ZIP SHA-256: ea38675219df7a087cc9cc5ddbfc370fc5e5652e966b2a716bb4e2c1dd551017
- This repository's original integration: src-tauri/experiments/lbppc_native.py and src-tauri/src/video.rs.

The original engine remains separately licensed by its authors. This launcher does not include game data or claim that upstream endorses the experimental profiles.
