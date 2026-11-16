# Upstream research — 2026-11-16

Research informs adapter behavior, not claims of game playability.

## LBPPC

- https://github.com/emurphy04/LBPPC/blob/main/README.md
- https://github.com/emurphy04/LBPPC/blob/main/launcher/lbppc_launcher.py
- https://github.com/emurphy04/LBPPC/blob/main/scripts/game_setup.py
- https://api.github.com/repos/emurphy04/LBPPC/releases/latest

Latest observed release v0.1.4. Windows ZIP includes LBPPC.exe. Official launcher source documents `--play` for direct launch and no arguments for its full settings UI. It stores configuration in `user/launcher.json`, including `game` and `update` keys. The adapter changes only those path keys, preserving other keys and a backup. Original game data is never written. Base data requires eboot.bin, base_001.farc and supported PARAM.SFO; update must match region and version. Version strings may have a leading zero (01.28). LBPPC does final preparation and validation.

## RPCS3

- https://github.com/RPCS3/rpcs3/blob/master/rpcs3/rpcs3.cpp
- https://github.com/RPCS3/rpcs3-binaries-win/releases/latest
- https://rpcs3.net/quickstart

Source documents `--no-gui`, `--fullscreen` and a positional (S)ELF path. The adapter passes the selected game's USRDIR/EBOOT.BIN as a separate process argument, with no shell. GUI/settings start without arguments. Firmware must be installed through RPCS3. Games are discovered from games.yml and dev_hdd0/game. The release-list endpoint was observed to return old records first, so the adapter deliberately uses `/releases/latest` (observed build 0.0.43-20606), not the first list item. Official archives are 7z with SHA-256 metadata.

## Native availability

- LittleBigRecomp https://github.com/benjidotsh/LittleBigRecomp describes macOS arm64 and Metal, not Windows. No official Windows release observed. Future release discovery requires a unique Windows archive; importing rejects non-PE/x64 executables.
- RePlanet author follow-up https://www.reddit.com/r/littlebigplanet/comments/1x9e57o/replanet_a_followup/ describes a private work-in-progress project. No public release source was verified. No repository was invented. RPCS3 remains the configured default.

## Visual study

Pod references showed a physical cardboard window, hanging planets, restrained ambient movement and spatial objects. Popit references showed compact floating panels, clear labels and material swatches. Original art here uses those broad material ideas without copying game art or layouts.

- Official LBP3 manual: https://playstation-doc.net/j/lbp3/ps4/screen.html
- Popit discussion and reference image: https://louisemclennan.wordpress.com/2016/11/30/top-5-favourite-video-game-interfaces/
- Reference-only Pod images found through image search; none distributed in this project.
