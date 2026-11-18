# Security

Patchwork Pod is an unofficial Windows launcher created by yasrax2. Security checks reduce risk; they do not establish that software is free of vulnerabilities or malicious upstream code.

Use GitHub's private vulnerability reporting feature on this repository for sensitive reports. Do not post tokens, account details, game files, private paths or exploit data in public issues. If private reporting is unavailable, publish only a non-sensitive request for a private reporting channel.

The launcher accepts downloads only from its fixed official GitHub projects over HTTPS, requires a matching SHA-256 checksum before installation, rejects archive traversal/links and limits actual extracted bytes. Imported executables remain programs the user chooses to trust. No executable is bundled with the launcher for games or emulators.

The frontend loads bundled code and local image data only. No remote webpage receives desktop permissions. Diagnostics omit private paths and account information. Local artwork and configuration are never uploaded by the application.

Before releases, run `npm audit`, `npm test`, `cargo test --manifest-path src-tauri/Cargo.toml --locked` and `python scripts/check_dependencies.py`. The last command checks the locked Windows graph against OSV. Non-Windows lockfile dependencies must be reviewed separately before another platform is supported. A clean scan only means the databases reported no matching advisories at that time.

Windows packages are currently unsigned. SHA-256 checksums identify the uploaded artifacts; they are not a code-signing identity or a guarantee of safety.

Optional LBPPC browser sign-in uses the supported upstream PKCE helper and a loopback callback. Helper source hashes are pinned for the supported engine revision. Windows DPAPI protects the remembered refresh session; setup errors returned to the UI omit provider responses and tokens. Do not copy engine account folders between Windows users. Imported engines are still executable code you choose to trust.

Optional source builds execute upstream build code at a pinned Git commit, after checking the official MSYS2 installer's SHA-256. Compiler packages are signed by MSYS2; current package versions and Python packaging dependencies are not fully pinned. Source-built engines are kept separate from existing engine files, and cancellation targets only the launched build process tree. Build logs can contain paths; review them before sharing.
