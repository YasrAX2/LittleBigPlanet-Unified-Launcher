# Troubleshooting

- **No native Windows download for LBP1/2:** LittleBigRecomp's observed build targets macOS arm64/Metal. RePlanet has no verified public release endpoint. Select RPCS3. Future native imports require an official Windows x64 executable; import support does not certify future game compatibility.
- **Version 1.5.0 PKG import:** use Install PS3 PKG for LBP1/2. PS4 packages are refused. Updates/DLC need a matching installed base game. Required licenses must come from your own copy. A processed package is not proof that a game is playable.
- **Version 1.5.0 firmware setup:** Prepare RPCS3 retries automatic installation. Checksum mismatch stops it; Install firmware file is the official local-PUP fallback. Custom RPCS3_CONFIG_DIR/VFS layouts may need setup in RPCS3.
- **Missing firmware:** Open RPCS3 settings and install official PS3 firmware using RPCS3's instructions and PlayStation's source. The launcher checks the standard dev_flash/vsh/module/vsh.self path. For custom firmware locations, use RPCS3 directly until its standard installation is configured.
- **Wrong folder / no PARAM.SFO:** Choose extracted game data, not an ISO, PKG or archive. For PS4 extractions, select the actual game root, not its parent wrapper. No decryption or game downloading is performed here.
- **Wrong LBP3 update:** The update region must match the base game, currently CUSA00473 or CUSA00063. A combined 01.28 folder is accepted; a lower base version needs a complete separate update. LBPPC performs final game validation and preparation.
- **Download failed / GitHub rate limit:** Retry after the network recovers or GitHub's limit resets. No token is requested or stored. Review upstream's release page if asset naming changes; ambiguous assets are rejected.
- **Hash mismatch:** No archive is installed. Retry. If it repeats, report the release/version to the upstream project.
- **Repair reports changed settings:** Customized configuration files can differ from the release. Repair preserves those files. Rename a broken engine setting manually after backing it up, or use the engine's own reset tools.
- **Cannot move/update an engine:** Close running games and engine windows. Managed operations reject junctions/reparse points and overlapping game-data folders. Imported copies are updated with their own upstream tools.
- **A game does not boot:** Successful import checks metadata and required paths; it cannot prove that all game data is decrypted, licensed or complete. Read the engine's error and official guide. No real-game playthrough was performed for this launcher release.
- **Configuration is unreadable:** Keep the corrupt file for diagnosis. In `%LOCALAPPDATA%\com.patchworkpod.launcher`, restore `config.backup.json` as `config.json` while the launcher is closed. No private paths are written into automatic crash reports.
- **Windows warns about an unknown publisher:** The release is unsigned; no signing certificate was available. Verify its SHA-256 against the published checksums and review source before deciding whether to run it.
- **Blank window:** Ensure the Microsoft WebView2 Evergreen Runtime is installed. Development builds also require the local Vite server; release builds do not.

Settings offers a locally saved diagnostics report. Inspect it before voluntarily attaching it to an issue. Never attach game dumps, firmware, private keys, account files or save data.

## No official checksum

Automatic installation refuses a release with no usable official SHA-256 checksum. Review the official project and use Import existing if you independently trust an installation. A checksum mismatch always stops installation.

## Report a problem

Include launcher/engine versions, game/title ID, exact error and whether Standard mode works. Settings can save a redacted local diagnostics summary. Inspect files before sharing. Do not upload game packages, firmware, licenses, account files or saves.
