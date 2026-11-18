# Version 1.3.0 verification

17 November 2026: 8 frontend tests and 30 Rust tests pass. Coverage includes unsupported FPS validation, legacy defaults, preservation of LBPPC account/control fields while synchronizing JSON/INI, RPCS3 custom/global profile merging and preservation of original files and Vblank timing.

The frontend production build passes. npm audit reports zero vulnerabilities; OSV reports zero advisories in 295 packages in the Windows target graph. A medium glib advisory remains in non-Windows dependencies; it is not dismissed or represented as fixed.

No gameplay FPS measurement or complete native visual/keyboard/contrast check was performed for this update. Desktop control was stopped during the earlier visual review. Selected FPS is an engine request, not a performance measurement. The Windows app is unsigned.

Updated packages embed the 14 approved default pictures; their inventory and hashes are checked by tests. Private version-history screenshots remain outside Git. See [video controls](VIDEO.md), [privacy](../PRIVACY.md) and [checklist review](CHECKLIST-REVIEW.md).

---

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

Packaged release smoke test: the compiled app loaded its bundled frontend at tauri.localhost and displayed all three library cards with the WebView2 network set offline. Both official engine settings windows also opened during desktop integration testing. This does not establish real-game offline playability.

Default-art refresh: the dependency audit now reports [GHSA-fc5q-39xc-vjpf](https://github.com/advisories/GHSA-fc5q-39xc-vjpf), an unpatched picomatch glob ReDoS affecting development tools. The audit accepts this advisory only for dependencies marked dev-only in the lockfile. Build/test globs are fixed in the repository; the packaged launcher includes no Node.js build tools and accepts no user globs. All other advisories still fail the build. Remove this exception when an upstream fix is available.

Appearance refresh verification: the approved 14 PNG hashes and PNG signatures pass. Backend tests confirm all default images are embedded and restoring the look preserves game paths, favorites and accessibility preferences. The main production frontend also passed a headless check with a synthetic default snapshot: three covers, background, ten palette icons, picture/color switching, restore defaults, 8 px blur, compact/wide navigation and no horizontal overflow. This frontend preview does not exercise native desktop IPC or gameplay.

PS4 requirement notice: both maintained versions now identify LBP3 as requiring the PS4 edition on its card, game page, file import screen and LBPPC download panel. Frontend tests and production builds pass. A headless check of the main production frontend confirmed the notice is visible before the folder picker, explicitly rejects PS3/PS3 PKG compatibility, and shows no PS3 PKG importer for LBP3. This is a wording change; LBP3 via RPCS3 remains unsupported and no game import or gameplay was tested.

RPCS3 YAML reader fix: synthetic PSID/IDPS tests preserve exact hexadecimal text, keep ordinary numeric/bool video fields typed, round-trip serialized profiles and reject malformed YAML without exposing private values. The opt-in local configuration test passed against the creator's existing RPCS3 configuration and verified its source bytes were unchanged. No private configuration or identifiers are included in the repository. LittleBigRecomp is no longer offered as a Windows engine. The About page and contributor files credit KodkTheMan (formerly shawarma1m8ly) for the original idea.
