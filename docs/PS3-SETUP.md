# PS3 setup implementation and limits

Version 1.6.0 adds LBP3 PS3 alongside LBP1/2. The launcher calls RPCS3's documented source-defined --headless --installfw and --headless --installpkg paths. It does not implement a second package extractor or supply game licenses.

## Firmware integrity

Sony's [HTTPS support page](https://www.playstation.com/en-us/support/hardware/ps3/system-software/) publishes an HTTP download link for firmware 4.93 whose path includes MD5 a2b60b6ac1d2e49e230144345616927c. During development that downloaded file matched the published MD5, and its SHA-256 was recorded as 158471fd834f8ea8036136b6aab43cd86c7ba73d79ca30e0af3c0fe0001cf365 (206197916 bytes). This SHA-256 is a launcher pin, not a claim that Sony publishes a SHA-256 manifest.

Runtime automatic setup accepts only the fixed URL, exact size and pinned SHA-256, without following redirects. RPCS3 then validates/installs the PUP. Changed/truncated content is refused. A future firmware version needs a reviewed pin update; the launcher does not trust arbitrary replacement files or disable certificate verification. Install firmware file accepts a local PUP through RPCS3's own validation.

## Completion and cancellation

RPCS3's CLI installation can return zero even when an operation fails, and the tested Windows build also produced an abnormal shutdown after explicitly logging successful firmware installation. Therefore completion requires a newly written RPCS3 log with its explicit operation success record. Firmware also requires the expected installed VSH file. PS3 package success is matched to the selected package's title ID, then the installed game metadata must match the selected LBP game before linking.

A local patchwork-setup-result.json records the operation, confirmation and shutdown exit code without private source paths. A nonzero shutdown is not hidden in that record. An incomplete-firmware marker prevents an interrupted launcher-managed firmware installation from being treated as ready. Retrying Prepare RPCS3 reinstalls it. Download cancellation stops the firmware operation and retains that incomplete state until a successful retry.

## Layouts and settings

Normal/portable RPCS3 data locations are supported. An external RPCS3_CONFIG_DIR is refused for automatic setup because its semantics cannot safely be guessed. Custom VFS paths may need setup through RPCS3. The LBP starting profile changes Write Color Buffers for game effects unless an explicit per-game choice already exists, and preserves other CPU, timing and global configuration. Its option can be disabled.

Package validation checks the PS3 signature/type, size and a safe content-ID title before RPCS3 receives a path. PS4 packages are refused. Original package files remain where selected; installed content is engine-owned. An update preserves an already linked disc-game folder instead of replacing it with the update-only directory. RPCS3 can leave an empty/partial directory after a rejected package; it is not linked as a playable game. Updates/DLC still require their base game. Real package installation and gameplay remain unverified because no owner game package was available.

## Primary implementation references

- [RPCS3 CLI parsing and headless installation](https://github.com/RPCS3/rpcs3/blob/master/rpcs3/rpcs3.cpp)
- [RPCS3 package/firmware installation and completion messages](https://github.com/RPCS3/rpcs3/blob/master/rpcs3/rpcs3qt/main_window.cpp)
- [RPCS3 settings definitions](https://github.com/RPCS3/rpcs3/blob/master/rpcs3/Emu/system_config.h)
