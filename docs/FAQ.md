# Questions and answers

### Can I use the PS3 version of LittleBigPlanet 3?

Yes, in **1.6.0**. Open LBP3, set **Game edition / engine** to **PS3 · RPCS3 emulator**, then import your PS3 game folder or choose **Install PS3 PKG**. The PS4 1.28 requirement applies only to LBPPC. Switching editions unlinks old file paths and resets video overrides; it does not delete files or saves.

[Complete guide](USAGE.md) · [Troubleshooting](TROUBLESHOOTING.md) · [Project](../README.md)

### Can I put a PKG in the game-folder picker?

No. A PKG is an installer file, not an installed game directory. In **1.6.0**, select a PS3 game (including LBP3 with RPCS3 selected) and click **Install PS3 PKG** instead. RPCS3 processes it and the launcher links a matching complete installed game. Older versions require installation in RPCS3 first.

### Does this support PS4 PKGs too?

No. The LBP3 / LBPPC option needs your complete extracted PS4 base-game folder and a matching 1.28 update. A PS4 package is rejected by the PS3 importer.

### What exact file do I select for a folder import?

Select the folder itself. PS3 editions need PARAM.SFO and USRDIR/EBOOT.BIN, directly or under PS3_GAME. LBP3 / LBPPC needs eboot.bin, base_001.farc and metadata under sce_sys (or a supported Sc0 layout). Keep the remaining files too. [Examples](USAGE.md#game-imports).

### Can I use ZIP, RAR, 7z or ISO?

Extract archive contents first. A PS3 ISO needs preparation through RPCS3's supported workflow before linking its folder here. Extraction alone does not decrypt games.

### Where do I get the games?

Use your own copies and upstream dumping/preparation instructions. This launcher does not download games, provide licenses or include game data.

### Why does LBP1/2 use RPCS3 if this is a PC launcher?

The supported Windows adapter for those games is emulation. LBP3 uses the separate native LBPPC project. A launcher does not itself turn PS3 code into a native Windows port.

### Will RPCS3 be ready automatically?

Version 1.5.0 downloads verified official firmware after engine installation/import and provides a per-game LBP starting profile. Prepare RPCS3 retries setup; Install firmware file is the local-PUP fallback. Your game must still be complete/playable, any required license must be yours, and hardware/driver compatibility matters. Custom RPCS3 data locations may need engine-managed setup.

### What is a PUP? Is it a game?

PS3UPDAT.PUP is PS3 firmware. It belongs in firmware installation, not the game-folder picker. It is downloaded from Sony separately and verified before automatic installation.

### Does a digital game need a RAP/license?

Some do. Install your own legitimate license through RPCS3. Patchwork Pod does not generate or supply it. Successfully importing metadata is not proof the game is licensed or boots.

### Can I import game updates or DLC?

PS3 updates/DLC can be processed by Install PS3 PKG after the matching base game is installed. Verify their details in RPCS3. LBP3's PS4 update uses the separate extracted-update folder selector; its title ID must match the base game.

### Which LBP3 version works?

Currently US CUSA00473 / Europe CUSA00063 with update 1.28. A combined 1.28 folder does not need a separate update. This adapter does not accept the PS3 edition of LBP3. Standard requirements refresh from the official LBPPC README.

### Are 60/120 native FPS guaranteed?

No. Standard LBPPC supports 30 native or 60 displayed using frame generation. Experimental 60/120 rendering profiles turn generation off but retain 30 Hz physics. Motion can still step at 30 Hz, and speed/stability remain unverified in real play. RPCS3 caps are not game unlocks. [Details](NATIVE-TESTING.md).

### Where are saves? Does Back up include them?

Saves belong to the engine. Launcher backups contain preferences/paths, not saves or artwork. Native rendering tests intentionally use separate test saves, accessible through Open test logs and saves.

### Why did the hamburger menu disappear?

The app shows full navigation in a wide window and a hamburger menu in a compact window.

### Can I change covers, background, blur and font?

Yes. Use Custom artwork on a game page and Settings for background, blur, colors, opacity and text. Current and LBP Wiki presets use the same licensed Rubik family with different heading weight. Local pictures stay on your PC.

### Will Unlink or Forget delete my game?

No. They remove references. Managed engine Uninstall retains its folder as a backup; game data is separate. Check retained backups before deleting them manually.

### Can it work offline? Does it send my files anywhere?

Configured engines can launch offline after setup. Release checks/downloads use the network; game online features belong to engines. Pictures and diagnostics are not automatically uploaded. Read the [privacy notice](../PRIVACY.md).

### The import succeeded but the game does not boot. Why?

Import validation checks metadata and paths, not every encrypted/missing/corrupt asset or game behavior. Inspect RPCS3/LBPPC's own error and [Troubleshooting](TROUBLESHOOTING.md). For a report, include versions, selected game/title ID, error text and whether it happens in Standard mode. Never attach game dumps, firmware, account files or license keys.

### Why does LBPPC need 1.28?

It supports that exact PS4 game version because its binary patches target that build. It is not a general requirement to use the latest game update. The separate RPCS3 option runs the PS3 edition and does not require PS4 version 1.28.
