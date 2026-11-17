# Patchwork Pod privacy notice

Updated 17 November 2026. Patchwork Pod is a free, unofficial Windows launcher created by **yasrax2**. This notice describes the launcher, not the independently installed engines or their online services.

## Information on your PC

The launcher stores preferences, selected game/engine paths, game identifiers, favorites, playtime, recent launcher activity and imported pictures in `%LOCALAPPDATA%/com.patchworkpod.launcher`. It also caches engine release information, compatibility requirements and download state. Pictures are copied locally and are not uploaded. The launcher has no account system, advertising or analytics SDK.

Configuration backups and exported files remain at the location you choose. Configuration backups contain local paths. Diagnostics contain engine versions, game identifiers and existence checks; they omit paths, usernames, artwork, logs and credentials. Nothing is sent automatically. Review any file before sharing it publicly.

## Network connections

Automatic requirements and installed-engine release checks contact official GitHub endpoints. Engine downloads contact their official GitHub release hosts. These services receive normal connection information, including your IP address, request headers and requested URL. The launcher does not append your game paths, artwork, account credentials or play history to these requests. GitHub describes its own handling in its [privacy statement](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement). External project links open your default browser and are governed by those sites' policies. The launcher uses no website tracking cookies or consent banner.

Installed games can be launched without release checks succeeding. Engine online features, accounts, telemetry and servers are controlled by the engine and its operators. Review their notices separately.

## Engine settings and backups

When you press Play, the LBPPC adapter updates the engine's local game paths. Optional video overrides also update selected video fields in `user/launcher.json` and `lbppc.ini`, preserving other fields and making local backups. Existing engine account fields remain in those files and their backups; the launcher does not send them to its developer. RPCS3 video overrides generate a separate launcher-owned profile from existing local settings. Treat engine folders and their backups as private.

## Remove your local data

Settings provides an **Open launcher settings folder** button. Close the launcher before deleting `config.json`, `config.backup.json`, `art`, `video`, cached requirements/release metadata or launcher download records. This resets preferences and removes local launcher copies of images. The **Remove local picture** and **Reset artwork** buttons unlink images; they do not securely erase stored copies or backups.

Do not delete the whole application folder blindly: its `engines` folder can contain installed engines, accounts and saves. Game data in other folders is not removed when launcher preferences are reset. You control separately exported backups, retained engine backups and any copies you shared. Standard deletion does not guarantee secure erasure.

There is no developer-hosted launcher user database or launcher account deletion service. For questions use the [public project](https://github.com/YasrAX2/LittleBigPlanet-Unified-Launcher); do not post private paths or credentials. Report vulnerabilities privately using the repository's Security page. If future versions add accounts, collection or new services, this notice must be updated before those features ship.
