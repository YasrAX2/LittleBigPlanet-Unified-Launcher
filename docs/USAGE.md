# Installation and use

Windows 10/11 x64. Run the setup EXE, or extract the portable ZIP and run `patchwork-pod.exe`. The installer supplies the WebView2 bootstrapper when needed. Portable users need Microsoft's WebView2 Evergreen Runtime, normally already present on supported Windows systems. No development tools are needed.

1. Open a game card. LBP1/2 default to RPCS3; LBP3 defaults to LBPPC.
2. Install the official engine, or import its existing Windows executable.
3. Choose your extracted, legally obtained game folder under Game files.
4. For LBP3, choose a separate matching update folder unless the base folder already contains supported version 1.28. US CUSA00473 and European CUSA00063 are supported by the current upstream requirements.
5. Validate, then Play. LBPPC retains its own graphics, controls and online settings. RPCS3 firmware and game-license installation remain in RPCS3.

LBPPC data folders must contain eboot.bin, base_001.farc and metadata under sce_sys or Sc0. Select the game folder itself; wrapper directories and incomplete system-only extractions should be opened at their actual game root. RPCS3 data needs PARAM.SFO and USRDIR/EBOOT.BIN, either directly or under PS3_GAME. The final engine may detect further missing or encrypted files.

## Maintenance

Downloads shows actual byte progress, speed and errors. Cancel retains a partial download. Retry resumes only when the server supports ranges and file identity can be validated. SHA-256 is checked when published by upstream. Archives are extracted into staging before switching installations.

Update / Repair reinstalls the current official release while retaining user-created files and customized JSON/INI/YAML/TOML/CFG settings. Verify compares release files to their recorded hashes; changed settings may appear as changed files. Roll back switches to the previous retained release and preserves current user files. Inspect engine-owned settings and saves before rolling back across major upstream changes.

Uninstall moves a managed engine to a timestamped sibling backup, retaining saves and all other files. It removes the launcher's active reference. Imported installations use Forget, which removes only the reference. Move copies and verifies a managed engine at the new location; the original remains as a backup. Old backup folders can consume disk space; remove them manually only after checking saves.

## Personalization and accessibility

Star cards for Favorites. Recently played and playtime are local. Playtime counts the launched engine's process lifetime, including its initial setup and open launcher window. Keep Patchwork Pod open until the engine exits.

Import your own PNG/JPEG artwork up to 8 MB. Artwork is private to this computer. Daytime, nighttime and Windows-following themes are in Settings. Text scales from 85% to 150%. Windows reduced-motion preferences are respected, with an additional explicit toggle.

Tab moves focus, Enter/Space selects. A detected controller uses D-pad/left stick, A to select and B to return. Physical controller hardware was not available for verification; support depends on WebView2 exposing the Windows controller.

## Offline and privacy

Configured local engines launch without a network requirement. Online game features belong to the engine. Release and requirements refreshes use GitHub and may fail offline without affecting saved configuration.

Backup exports launcher preferences and local file paths. Keep backups private. It does not back up game saves or artwork. Restore retains a backup of the current configuration and imports engine folders as user-owned references for safety. Diagnostics omit local paths, names, logs, artwork and account credentials. Nothing is uploaded automatically.
