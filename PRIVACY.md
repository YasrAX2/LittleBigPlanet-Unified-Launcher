# Patchwork Pod privacy notice

Updated 17 November 2026. Patchwork Pod is a free, unofficial Windows launcher created by **yasrax2**. This notice describes the launcher, not the independently installed engines or their online services.

## Information on your PC

The launcher stores preferences, selected game/engine paths, game identifiers, favorites, playtime, recent launcher activity and imported pictures in `%LOCALAPPDATA%/com.patchworkpod.launcher`. It also caches engine release information, compatibility requirements and download state. Custom imported pictures are copied locally and are not uploaded. Default artwork is bundled publicly with the application. The launcher has no account system, advertising or analytics SDK.

Configuration backups and exported files remain at the location you choose. Configuration backups contain local paths. Diagnostics contain engine versions, game identifiers and existence checks; they omit paths, usernames, artwork, logs and credentials. Nothing is sent automatically. Review any file before sharing it publicly.

## Network connections

Automatic requirements and installed-engine release checks contact official GitHub endpoints. Engine downloads contact their official GitHub release hosts. These services receive normal connection information, including your IP address, request headers and requested URL. The launcher does not append your game paths, artwork, account credentials or play history to these requests. GitHub describes its own handling in its [privacy statement](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement). External project links open your default browser and are governed by those sites' policies. The launcher uses no website tracking cookies or consent banner.

Installed games can be launched without release checks succeeding. Engine online features, accounts, telemetry and servers are controlled by the engine and its operators. Review their notices separately.

## PS3 setup and local packages

Version 1.5.0 downloads the pinned official PS3 firmware from Sony's published endpoint and verifies its SHA-256 before RPCS3 checks/installs it. Sony receives ordinary connection information. A PS3 PKG you select is passed locally to RPCS3; game packages and game licenses are never uploaded by the launcher. Installed games remain in RPCS3 storage. A generated LBP starting profile preserves original engine configuration files and custom per-game color-buffer choices.

## Engine settings and backups

When you press Play, the LBPPC adapter updates the engine's local game paths. Optional video overrides also update selected video fields in `user/launcher.json` and `lbppc.ini`, preserving other fields and making local backups. Existing engine account fields remain in those files and their backups; the launcher does not send them to its developer. RPCS3 video overrides generate a separate launcher-owned profile from existing local settings. Treat engine folders and their backups as private.

## Experimental native rendering profiles

The optional native tests run offline through the official engine's script interface. Separate native-tests/60 and native-tests/120 folders hold test saves, prepared cache, graphics settings and local engine logs. Existing engine accounts are not loaded. Logs can contain details from your local engine; inspect them before sharing. Nothing is automatically sent to the launcher developer.

## Remove your local data

Settings provides an **Open launcher settings folder** button. Close the launcher before deleting `config.json`, `config.backup.json`, `art`, `video`, `native-tests`, cached requirements/release metadata or launcher download records. This resets preferences and removes local launcher copies of imported images. Bundled default pictures remain available in the application. The **Remove local picture** and **Reset artwork** buttons unlink images; they do not securely erase stored copies or backups.

Do not delete the whole application folder blindly: its `engines` folder can contain installed engines, accounts and saves. Game data in other folders is not removed when launcher preferences are reset. You control separately exported backups, retained engine backups and any copies you shared. Standard deletion does not guarantee secure erasure.

There is no developer-hosted launcher user database or launcher account deletion service. For questions use the [public project](https://github.com/YasrAX2/LittleBigPlanet-Unified-Launcher); do not post private paths or credentials. Report vulnerabilities privately using the repository's Security page. If future versions add accounts, collection or new services, this notice must be updated before those features ship.

## Optional LBPPC browser sign-in and source builds (1.7.0)

Pressing Sign in / Create account contacts the official LBPPC identity service in your browser. Registration, passwords, verification and server privacy rules belong to that service. Patchwork Pod runs the installed, verified official sign-in helper with PKCE and a local callback. Only the engine keeps a Windows DPAPI-protected refresh session, under `user/launcher.json`. Access tokens remain in the helper/engine process; the UI receives only account name, saved-session status and sanitized errors. Launcher backups and diagnostics do not include this session. Sign out removes this browser session locally and requests revocation; it does not erase your website account or unrelated engine settings.

The optional source build downloads official MSYS2 tools/packages, GitHub source/submodules and Windows Python packaging dependencies. Python may be installed per-user through winget when absent. Build tools/source/cache stay in the selected build folder; compiler logs stay in the local launcher builds folder and may include your local paths. Review logs before sharing. Successful builds copy existing engine user data to a new local engine folder and retain the old folder, so both copies may contain protected account data and saves. No account data is uploaded by the build.
