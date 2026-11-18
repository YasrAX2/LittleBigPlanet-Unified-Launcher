# LBPPC source build, game setup and online accounts

These controls are in **Downloads > Build and set up LBPPC** in Patchwork Pod 1.7.0. They apply to LBP3's PS4 port. LBP1, LBP2 and LBP3's PS3 edition use RPCS3 separately.

## Easiest option

Press **Install LBPPC** to use the official ready-made Windows engine. You do not need to build source to play. You still need your own complete extracted PS4 base game and matching 1.28 content.

## Build on this PC

1. Press **Build LBPPC from source** and read the setup details.
2. Leave at least **20 GB free**. First setup downloads several GB and can take a long time.
3. Choose **Build from source**, or **Choose a different build folder** if your Windows profile has spaces/non-ASCII characters. Choose an ordinary local folder with a short ASCII path and no spaces, links or junctions.
4. Watch **Downloads**. **Open build logs** opens the local log folder. **Cancel** stops this build and its child tools; retry reuses cached tools and source.
5. After compilation and packaging succeed, the launcher switches to a separate completed engine. Existing engine files stay on disk; settings and saves are copied to the new engine. Do not delete the old engine until you verify your saves.

The build uses official LBPPC v0.1.5 at commit `3a3c2111bb0a137f1c564ccd2f16e357393ac059`, a SHA-256-verified official MSYS2 installer, and signed compiler packages. Tools live in `%LOCALAPPDATA%\PatchworkBuild` by default. Windows Python is installed for your account through winget only if missing. This operation does not require adding developer tools to the system PATH. Current compiler package versions can change; upstream incompatibilities or network failures can still stop a build.

Source-built engines are shown as user-managed installations. Their files are not automatically overwritten by the normal release updater. Optional NVIDIA DLSS components are not downloaded by this build; use the official packaged release if those components are required. Building the runtime does not add game assets or implement native 60/120 Hz gameplay.

## Set up game files

1. Select LBP3's **PS4 / LBPPC** edition.
2. Link the complete extracted base-game folder and matching update folder. A combined folder is supported when it contains all required files. [Import guide](USAGE.md#game-imports).
3. In Downloads, press **Check and set up game**. The official LBPPC checker reports missing content and can detect an adjacent update. Complete files are applied to the engine settings while preserving other settings.
4. Press **Play**. The engine performs its normal preparation and launches the game.

A version number alone does not prove completeness. An update-only PKG, encrypted retail package or missing `eboot.bin`/FARC content cannot be repaired by the source-build button. The launcher does not download games, licenses, decryption keys or unofficial packages.

## Account creation and automatic sign-in

1. Install or build the supported LBPPC version.
2. Press **Sign in / Create account**. The official LBPPC sign-in page opens in your browser.
3. Sign in there, or use its registration flow. Enter passwords only on the official page. Account verification, terms and any other server requirements must be completed by you.
4. Return to the launcher. It shows **Saved account** after a successful callback. The session is protected by your Windows account and saved in the engine's own `user/launcher.json`; passwords and tokens are not returned to the launcher UI or included in launcher configuration backups.
5. Play normally. LBPPC uses its saved session and refreshes it when needed. An expired/revoked session, another Windows account or unavailable server may require signing in again.

**Cancel sign-in** ends the pending browser flow; it does not delete a website account. **Sign out** removes the saved LBPPC browser session locally and asks the service to revoke it. It does not log other browser tabs out or erase unrelated legacy account settings. A saved account indicator is local status, not a live server connection test.

This uses the official community service `play.lbppc.com`. Friends, invitations and joining a co-op session are handled in LBPPC's own settings/game interface. Patchwork Pod does not open router ports or install a multiplayer server. Your friend needs compatible game files and their own account. RPCN/Beacon for RPCS3 is a separate setup.

## Launcher does not open

Earlier builds could leave an invisible process after closing during a download or running game. Version 1.7.0 prevents the window from closing while work is active and restores minimized windows on a second launch. If an older version is already stuck, verify no game/setup is running, end that old **patchwork-pod.exe** in Task Manager, then open the updated shortcut.

## Verification limits

Automated checks cover protected account storage, cancellation, unchanged unrelated settings, malformed-file refusal and file-check results. Local installed-engine status is checked separately. Browser registration, live server login and two-player multiplayer require real user interaction and game testing; passing storage tests does not prove them. See [PC project research](PC-PORTS-RESEARCH.md) and [privacy](../PRIVACY.md).
