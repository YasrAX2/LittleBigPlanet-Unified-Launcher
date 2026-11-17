# Video settings

Version 1.3.0. Checked against upstream source on 17 November 2026.

Open Settings, then Video settings, then a game, or open a game from Library. Enable **Use these video settings when launching**. Overrides are off by default; existing engine preferences remain in control until enabled. Changes apply on the next Play.

| Engine | Controls | FPS support |
| --- | --- | --- |
| LBPPC / LBP3 | Resolution, display mode, FSR 1 upscaling, frame generation | Keep existing, 30 native, or 60 displayed with FSR 3.1 / supported DLSS hardware. 120 is disabled because upstream has not implemented it. |
| RPCS3 / LBP1 and LBP2 | Resolution scale, fullscreen, frame limit | Keep existing, 30, 60 or 120 cap. A cap does not unlock game code. Higher rates require a compatible patch and sufficient hardware. Vblank and CPU settings are preserved. |

LBPPC simulates 30 game steps per second in its 60 FPS frame-generation mode. Native 60/120 FPS is upstream future work. See the [official LBPPC README](https://github.com/emurphy04/LBPPC). A selected rate is not measured performance.

LBPPC overrides update documented launcher JSON keys and `frame_gen` in `lbppc.ini`, preserve unrelated account/control settings and save local backups. RPCS3 overrides copy global and game-specific settings into an application-owned YAML profile passed with `--config`. Original RPCS3 files are not overwritten. Current `config/`, legacy and `portable` layouts are supported. If `RPCS3_CONFIG_DIR` is set without a portable folder, overrides refuse to guess; disable them and use RPCS3 settings.

RPCS3 references: [CLI option](https://github.com/RPCS3/rpcs3/blob/master/rpcs3/rpcs3.cpp), [settings keys](https://github.com/RPCS3/rpcs3/blob/master/rpcs3/Emu/system_config.h), [configuration loading](https://github.com/RPCS3/rpcs3/blob/master/rpcs3/Emu/System.cpp), [profile location](https://github.com/RPCS3/rpcs3/blob/master/rpcs3/Emu/system_utils.cpp).

No patches or game files are bundled. Configuration preservation and validation are tested. Actual game performance needs testing with your own lawful game files and hardware.
