# Verification record

Tested on Windows x64, 2026-11-16. No commercial game data was used and no real game playthrough is claimed.

## Passed

- 16 Rust unit and filesystem tests: default games, configuration persistence, invalid configuration rejection, SFO parsing and truncation, missing/wrong game metadata, non-Windows executable rejection, official Windows asset selection, upstream requirement parsing, redirect URL validation, real SHA-256 hashing, archive traversal rejection and valid extraction, preservation of user files, retained uninstall, documented launch options and LBPPC account/settings preservation.
- 3 TypeScript tests: catalog completeness, byte-based progress and state, playtime/size formatting.
- 22 desktop integration checks using the actual running Tauri WebView2: navigation, all three games, layout at 1280×720/1920×1080/2560×1440, 150% text size, theme persistence, missing-path rejection, honest unpublished RePlanet status, live requirements, official release discovery, LBPPC and RPCS3 download/install and installed-file hash verification.
- Real official LBPPC v0.1.4 and RPCS3 0.0.43 Windows downloads installed from GitHub, verified, repaired and rolled back. Both engines' original GUI windows opened successfully through the launcher. Firmware and games were not installed.
- Synthetic PS3 metadata import, a separately compiled Windows fixture process launch, running-state cleanup, local playtime tracking and original-file preservation.
- Automated WCAG A/AA scans across six sections in both themes: zero reported violations after contrast fixes. This does not replace a human accessibility audit.
- Frontend production build, local Windows NSIS packaging, Windows GitHub Actions build and tests.
- npm dependency audit: zero reported vulnerabilities. Rust dependency license inventory is in RUST-DEPENDENCIES.txt; a RustSec advisory audit was not run.

Machine-readable results accompany this document. Screenshots are from the actual desktop application's WebView2, not generated mockups. Local paths are masked in screenshots where needed. Earlier extended-test results retain the initial contrast finding; accessibility-results.json records the final corrected twelve-screen pass.

## Limits

No owned game dumps, firmware, live online accounts or physical game controller were available for gameplay/hardware tests. Controller navigation is implemented but hardware-unverified. Text scaling was tested; actual Windows display-DPI changes were not made. Offline configuration and local execution are independent of release discovery, but no complete real-game offline session was tested. Future native adapters are compatibility hooks, not verified ports. Installer signing requires a certificate and is not included.

## Reproduce

Run `npm test`, `npm run build` and `cargo test --manifest-path src-tauri/Cargo.toml --locked`. Build a debug app with `PATCHWORK_TEST_HOME` set to an isolated writable directory and WebView2 debug arguments `--remote-debugging-port=9223`; run `npm run dev` and start the app. `node tests/desktop.mjs` performs real engine downloads; it requires Internet access and disk space. `node tests/visual-check.mjs` captures all sections and runs accessibility scans. `tests/process-check.mjs` expects a separately built two-second fixture executable under the adjacent fixtures/engine directory. These test/debug settings are not enabled in shipped release configuration.
