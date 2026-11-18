# Complete launcher guide

[Project](../README.md) · [Q&A](FAQ.md) · [Troubleshooting](TROUBLESHOOTING.md)

## Install and choose a version

Open [Releases](https://github.com/YasrAX2/LittleBigPlanet-Unified-Launcher/releases). Stable 1.3.0 has standard video settings. 1.4.0 adds experimental native-rendering profiles and Wiki typography; 1.5.0 also adds PS3 package imports and automatic firmware preparation.

Choose the **x64-setup.exe** installer, or extract the full **Windows-x64.zip** and run `patchwork-pod.exe`. The Source ZIP is not a ready-to-run application. Portable preferences still live in Windows local application data. No development tools are required. WebView2 is required; the installer can install its bootstrapper. Builds are unsigned. Compare downloads against the release's SHA256SUMS if checking integrity.

Engines have separate hardware requirements. [LBPPC's official requirements](https://github.com/emurphy04/LBPPC) include Vulkan graphics support. Installation and metadata checks cannot guarantee that a game runs on every PC.

## Engines and first-time PS3 setup

Open a game in Library, then **Install** its engine, or **Import existing** and choose its already extracted Windows executable:

| Game | Engine executable |
| --- | --- |
| LBP1 and LBP2 | `rpcs3.exe`; both games can share the same engine. |
| LBP3, PS4 edition | `LBPPC.exe`; keep its complete package, including bin/scripts and companion files. |

Keep engine folders and game data separate, for example `Engines/RPCS3` and `Games/LBP1`. Do not put standalone game dumps inside managed engine directories. RPCS3-installed digital games are the exception: they live in its dev_hdd0/game storage and are relinked if the managed engine moves.

Version 1.5.0 prepares official PS3 firmware automatically after installing/importing RPCS3. **Prepare RPCS3** retries setup. **Install firmware file** accepts an official local `.PUP` if automatic download is unavailable. Installation runs through RPCS3's supported headless installer and checks for the installed firmware afterward. The bundled code pins firmware 4.93 by its size and SHA-256. Sony's published HTTP download is authenticated against that pin before installation; mismatches are refused. Firmware is downloaded separately, not redistributed in release assets.

Get firmware only from [PlayStation's official page](https://www.playstation.com/en-us/support/hardware/ps3/system-software/). Follow the [RPCS3 quickstart](https://rpcs3.net/quickstart) for custom configurations and game licenses. An unusual RPCS3_CONFIG_DIR or custom virtual filesystem can require setup in RPCS3 itself. Normal and portable layouts are recognized.

LBP1/2 currently use emulation through RPCS3 on Windows. Their future-native-project import options do not establish present Windows game compatibility. In 1.6.0, LBP3 offers PS3 through RPCS3 or PS4 through the separate native LBPPC project. Choose the edition on its game page before importing.

## Game imports

Open **Game files**, select a game and use **Choose game folder**. Select the actual root directory containing the files below. Keep all remaining dump files; these trees show required markers, not complete file lists.

### LBP1 / LBP2 / LBP3: extracted PS3 games

```text
My LBP1/                 <- choose this folder
  PS3_GAME/
    PARAM.SFO
    USRDIR/
      EBOOT.BIN
      ...other game files...
```

Selecting `PS3_GAME` itself also works. Digital installations may use this direct layout:

```text
Installed game/         <- choose this folder
  PARAM.SFO
  USRDIR/
    EBOOT.BIN
    ...other game files...
```

A lone EBOOT.BIN or PARAM.SFO is insufficient. Dump preparation, any required decryption and game licenses belong to the engine's supported workflow.

### PS3 PKG files: LBP1/2 in 1.5.0, plus LBP3 in 1.6.0

1. Install/import RPCS3 first.
2. On the matching game's Game files panel, click **Install PS3 PKG**.
3. Select your own `.pkg` file. The launcher checks the PS3 header, content ID and complete declared size; PS4 packages are rejected.
4. Firmware is prepared if missing. RPCS3 installs the package; large files take time.
5. A matching complete LBP game folder is linked automatically when found. Use **Validate**, then **Play**.

Updates and DLC require their base game already installed. When a matching installation already existed, the launcher reports package processing and linking; check RPCS3 for update/DLC details. It does not claim a zero process exit alone proves successful installation. A missing/wrong installed game is reported instead of silently linking unrelated data.

Some digital games need the user's own license. The launcher cannot generate it. Install your legally obtained license through RPCS3. Original packages are retained. If an installer fails, inspect RPCS3's local log before retrying.

### LBP3 / LBPPC: extracted PS4 base game and update

```text
My LBP3/                <- base game folder
  eboot.bin
  base_001.farc
  sce_sys/
    param.sfo
  ...other game files...

My LBP3-Update/         <- separate matching update folder
  eboot.bin
  sce_sys/
    param.sfo
  ...other update files...
```

Current supported title IDs are US **CUSA00473** and Europe **CUSA00063**, update **1.28**. Choose the matching separate update unless the base folder is already combined at 1.28. Regions must match. Metadata at `Sc0/param.sfo` is also recognized in certain layouts, but a system-only Sc0 folder is not a complete base game.

A PS3 copy of LBP3 is not accepted by this PS4 adapter. PS4 PKG installation is not implemented. Prepare your own extracted game/update using the [official LBPPC guide](https://lbppc.com/getting-started) first.

### Other formats

| Format | Treatment |
| --- | --- |
| ZIP / 7z / RAR | Extract fully, then choose a usable game root. Extraction does not decrypt a game. |
| PS3 ISO | Follow RPCS3's supported preparation workflow to obtain folder data before linking here. |
| PS3 PKG | Use Install PS3 PKG in 1.5.0; older launcher versions require installation in RPCS3 first. |
| PS4 PKG | Not supported directly. Prepare the extracted PS4 game/update first. |
| BIN / SFO / FARC | Components inside a complete game; do not select them individually. |
| Engine EXE | Use Import existing in the engine panel. |
| PNG / JPEG | Use artwork/background selection, not game import. |

## Validate, launch and navigate

After linking, press **Validate** and read missing-file/version/region messages. Return to the game's page and press **Play**. First LBPPC preparation, shader compilation and RPCS3 caches can take time. **Change folder** relinks moved data; **Unlink** removes its launcher reference without deleting it. Folder discovery finds possible installed games; inspect results before linking.

| Page | Use |
| --- | --- |
| Home | Open games and return to recent activity. |
| Library | Browse game details and star Favorites. |
| Downloads | Install/import engines and maintain managed installations. |
| Game files | Link/validate games and updates, import PS3 packages or discover folders. |
| Settings | Appearance, video shortcuts, text, controls, storage, backups and diagnostics. |
| About | Creator, project/engine links, licenses and privacy. |

Wide windows have full navigation. Compact windows show the hamburger menu, which disappears when expanded. Playtime is local and counts the launched engine's lifetime, including preparation/open engine windows. Keep Patchwork Pod open until the engine closes.

## Video and LBP settings

Open **Video settings** on a game's page and enable overrides to apply them on the next Play. Standard video overrides are off by default.

| Mode | Controls and limitations |
| --- | --- |
| LBPPC Standard | Resolution, display mode, FSR 1, keep engine FPS, 30 native or 60 displayed through supported frame generation. |
| RPCS3 | Fullscreen, resolution scale and 30/60/120 caps. Caps alone do not unlock game FPS. |
| LBPPC native rendering test | 60/120 targets, generation off, unmodified official 0.1.5 only, offline and separate test saves. Physics remains 30 Hz. |

Version 1.5.0 also provides an **LBP starting profile** for RPCS3 color-buffer effects. Existing per-game color-buffer choices are respected, global configuration files remain intact and the profile can be disabled on the game page. CPU settings and Vblank timing are preserved. This is a starting configuration, not a guarantee of gameplay compatibility or performance.

Experimental rendering does not implement 60/120 Hz simulation or world-motion interpolation. Character motion may still step at 30 Hz, and missed targets can change game speed. Read [native test instructions](NATIVE-TESTING.md) before testing; start with 60. Other graphics, controls, audio and accounts remain in **Open engine settings**.

## Appearance and controls

Settings includes theme-color/local-picture backgrounds; 0–24px picture blur; black/green and nine optional color pairs; panel opacity; Daytime/Nighttime/Follow Windows; Current/LBP Wiki text presets; 85–150% text scale; reduced motion and controller navigation. Wiki typography uses the existing licensed Rubik font with heavier headings.

Game pages offer Custom artwork. Backgrounds/covers accept PNG/JPEG up to 8 MB and stay local. Reset/remove controls unlink artwork rather than securely erasing stored copies. Public builds include the selected default background, covers and Popit icons, but no proprietary game font files. **Restore default look** resets appearance without changing game paths or accessibility settings.

Keyboard: Tab moves focus, Enter/Space activates. Optional controller navigation uses D-pad/left stick, A to select, B to return; Windows/WebView2 must expose the controller. Configure actual in-game mappings in the engine. Physical controller behavior and complete native accessibility checks remain unverified.

## Downloads and maintenance

Downloads show real progress, speed and errors. Cancel retains partial files; retry resumes only when server identity/range checks permit it. Automatic engine installation requires official SHA-256 verification.

| Action | Behavior |
| --- | --- |
| Update / Repair | Reinstalls release files while retaining the previous engine and user-created/custom settings. |
| Verify files | Compares recorded hashes; customized settings may legitimately differ. |
| Move | Copies/verifies a managed engine; retains the original backup. |
| Roll back | Uses the retained previous release; check upstream save/settings compatibility. |
| Uninstall | Retains a managed installation as a timestamped sibling backup and removes its active reference. |
| Forget | Removes an imported engine reference; its files stay user-managed. |

Change Engine location affects new installations. Move existing managed engines through Downloads. Close running engines before maintenance. Backups consume disk space; inspect saves before manually deleting them.

## Saves, backups and diagnostics

Saves belong to their engines. **Launcher backups do not include game saves or artwork.** Use engine save tools while closed. Native-rendering tests use separate local native-tests/60 and native-tests/120 folders; use **Open test logs and saves**.

Settings offers Back up (preferences/paths to JSON), Restore (retains current config and references restored engines as user-owned), Save diagnostics (local version/existence summary without paths/usernames/logs/credentials) and Open launcher settings folder. Revalidate restored paths and choose custom artwork again. No reports are automatically uploaded.

Local data lives in `%LOCALAPPDATA%\com.patchworkpod.launcher`. Do not blindly delete the whole folder: default engines inside it may contain saves/accounts. Review [privacy and deletion instructions](../PRIVACY.md). Before downgrading from a native-test version to 1.3.0, select Standard rendering and save.

Configured games can launch offline once prerequisites exist. Initial downloads need a network connection; engine online services have their own accounts/requirements. [Troubleshooting](TROUBLESHOOTING.md) · [Common questions](FAQ.md).

Technical setup details and verification limits: [PS3 setup](PS3-SETUP.md).

## LBP3 PS3 support in 1.6.0

Open **Library > LittleBigPlanet 3 > Game edition / engine > PS3 · RPCS3 emulator**. Install/import RPCS3 and prepare its official firmware, then select a complete extracted PS3 game folder or use **Install PS3 PKG**. Game licenses remain user-supplied in RPCS3. PS4 packages are rejected. **Discover games** also finds LBP3 in RPCS3. Switching editions resets only linked game/update metadata and video overrides; favorites, artwork, playtime and physical files are preserved. Link that edition again after switching back. PS3 saves stay in RPCS3; they are separate from the PS4 port. The PS4 1.28 update requirement does not apply to RPCS3. Emulator compatibility and FPS depend on the game, settings and hardware.
