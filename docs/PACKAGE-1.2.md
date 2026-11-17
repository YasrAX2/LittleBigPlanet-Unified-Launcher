# Windows package verification

The final local installer completed with exit code 0. Its installed application was tested at the packaged `http://tauri.localhost/` origin, independently of Vite.

- Default black/green, cover and backdrop loading, Rubik loading, optional theme persistence and offline assets passed: packaged-popit-results.json.
- Native compact/maximize/restore checks passed on the installed application: native-window-results.json. Compact width was 1004 CSS pixels; maximized width was 1920. An open drawer closed on expansion.
- Five frontend tests and nineteen Rust tests passed. reference-popit-results.json records six layout sizes and 24 automated accessibility scans with zero violations.
- The installed binary and portable binary differ only in the expected three-byte Tauri bundle marker: `__TAURI_BUNDLE_TYPE_VAR_NSS` versus `__TAURI_BUNDLE_TYPE_VAR_UNK`. Every other byte matches. Tauri patches this marker for the installer: https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle.rs .
- The temporary installation was removed. The existing user configuration was restored byte-for-byte after the appearance persistence test. No game files or installations were changed.

The Windows package is unsigned. Gameplay and physical controller behavior were not retested for this appearance update. Automated accessibility results do not establish a complete audit.
