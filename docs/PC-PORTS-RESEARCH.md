# LittleBigPlanet on PC: project research

Checked 18 November 2026. These are independent community projects, not Sony PC releases. A development video, public source repository and downloadable playable build are different milestones. Project authors' gameplay claims below were not independently reproduced here.

## What can I use in this launcher?

| Game | Windows route supported by Patchwork Pod | Native PC project status |
| --- | --- | --- |
| LittleBigPlanet | PS3 edition through RPCS3 | LittleBigRecomp targets Apple Silicon Macs. A separate Windows project has been demonstrated, but no verified public build was found. |
| LittleBigPlanet 2 | PS3 edition through RPCS3 | Multiple native projects exist. None checked was a complete, ready-to-play Windows replacement. |
| LittleBigPlanet 3 | PS3 edition through RPCS3, or PS4 edition through LBPPC | LBPPC has a published Windows build and documented source-build process. |

RPCS3 is an emulator. LBPPC runs PS4 game code through a PC runtime with reimplemented console services and GPU translation. Static recompilation translates console executable code ahead of time. These approaches are not interchangeable, and their game files, saves and multiplayer services differ.

## LBP1

**[LittleBigRecomp, by benjidotsh](https://github.com/benjidotsh/LittleBigRecomp)** is a static recompilation project for the PS3 BCES00141 v02.00 edition. Its documented host is macOS on ARM64, with Metal, Xcode, SDL2 and local game extraction. The repository describes early boot and Introduction progress. No official Windows release was found on its [releases page](https://github.com/benjidotsh/LittleBigRecomp/releases). Building this particular source on Windows does not make its Mac-specific runtime a Windows port.

**[Ende18's Windows demonstration](https://www.reddit.com/r/decomps/comments/1x97rgo/early_look_littlebigplanet_ps3_pc_port/)** describes native x86-64 recompilation, a Vulkan renderer, native SPU replacements and render interpolation while simulation remains 30 Hz. The author reports testing story levels and says a GitHub release is planned. The checked announcement does not provide a released build. It is a separate project from LittleBigRecomp; this launcher does not install an announcement as if it were a working engine.

PSP/Vita recompilation projects concern different games. Their files cannot replace the PS3 game's engine or data.

## LBP2

**[RePlanet, by reallyamusedd](https://www.reddit.com/r/littlebigplanet/comments/1x9e57o/replanet_a_followup/)** is a native x86-64 project based on ps3recomp. The author names Windows and DirectX 12, and reports connecting to Beacon using RPCN. The checked follow-up says the project is private, with a possible playtest rather than a promised release date. A public, verifiable Windows download was not found in this review. Discord-only development was not independently inspected.

**[ps3recomp-lbp2, by zeropaulsy](https://github.com/zeropaulsy/ps3recomp-lbp2)** provides public research source for USA BCUS98245. Its 17 November status reports boot, a scripted opening-level walkthrough and the next level loading. Building requires decrypting and lifting one's own PPU executable and SPU jobs, including manual steps. Its examples use shell scripts and Linux audio isolation. Reported performance includes the normal 30 FPS cap and roughly 36–41 FPS uncapped in the opening level, with slower cutscenes. Online is stubbed; rendering and pacing issues remain. The checked [releases page](https://github.com/zeropaulsy/ps3recomp-lbp2/releases) has no packaged release. Public source does not establish turnkey Windows support.

**[craftworld-rs, by chilladept](https://github.com/chilladept/craftworld-rs)** has [Windows preview downloads](https://github.com/chilladept/craftworld-rs/releases). It is an independent Rust/Bevy asset explorer and approximate runtime: resources, levels, playable Sackboy and partial script execution. Its own README explicitly describes experimental, incomplete compatibility. It requires the user's game data; it is not the complete LBP2 game and is not automatically substituted for RPCS3 here.

**Answer:** LBP2 PC work is real. A complete public Windows port equivalent to installing the original game was not verified. RPCS3 remains this launcher's supported Windows route for LBP2.

## LBP3

**[LBPPC, by emurphy04 and contributors](https://github.com/emurphy04/LBPPC)** has a [v0.1.5 Windows release](https://github.com/emurphy04/LBPPC/releases/tag/v0.1.5). Its [Windows build guide](https://github.com/emurphy04/LBPPC/blob/v0.1.5/packaging/windows/README.md) uses MSYS2 CLANG64, compiler dependencies, submodules, `build.sh` and Windows Python packaging. Patchwork Pod's source-build button follows this process at commit `3a3c2111bb0a137f1c564ccd2f16e357393ac059`. Compiler packages come from signed MSYS2 repositories; their current versions can change, so this is not a fully reproducible toolchain lock.

LBPPC needs complete, extracted PS4 CUSA00473 (US) or CUSA00063 (EU) data with matching **1.28** content. A PS3 PKG, an update by itself, or a folder that only reports version 1.28 does not satisfy this requirement. A runtime build cannot supply, decrypt or repair missing game assets. LBP3's PS3 edition is a separate RPCS3 option without a PS4 update requirement.

**[LittleBigPlanet-3-Recompiled, by PurpleGitty](https://github.com/PurpleGitty/LittleBigPlanet-3-Recompiled)** is another public PS4-derived runtime project. Its README describes Windows build requirements and early Pod/Prologue/Create progress. No packaged release was verified. It has not been tested here and is not assumed compatible with the LBPPC adapter.

## Multiplayer and accounts

LBPPC's [browser sign-in helper](https://github.com/emurphy04/LBPPC/blob/v0.1.5/scripts/lbppc_signin.py) implements OAuth with PKCE, a local callback and cancellation. Patchwork Pod calls this official helper, so account creation/sign-in stays on the official browser page. It stores the returned refresh session under the same Windows DPAPI protection used by LBPPC. Status checks do not refresh or rotate tokens. The engine refreshes its own session when playing; an expired or revoked session can require signing in again.

LBPPC's [online guide](https://github.com/emurphy04/LBPPC/blob/v0.1.5/docs/online.md) describes Refresh for profiles/levels and shadNet for friends and multiplayer signaling. The supported default is the existing community service; this launcher does not host a server, change router settings or create accounts without the user completing the browser step. Friends, invitations and joining remain in LBPPC. RPCS3/RPCN accounts are separate.

Local protected-storage and setup tests are separate from successful live account creation, server access, two-player co-op or complete-game compatibility. Those require real accounts, complete game data, compatible hardware and runtime testing.

## FPS limits

Higher caps do not rewrite game timing. LBPPC's standard runtime retains 30 Hz simulation; frame generation can raise displayed output. Patchwork Pod's separate offline 60/120 native-rendering experiments do not establish correct 60/120 gameplay, world interpolation or faster simulation. No such guarantee is made for these projects or for RPCS3 patches. See [video settings](VIDEO.md) and [native test limits](NATIVE-TESTING.md).

## Research method and limits

Checked project READMEs, author announcements, release listings and upstream Windows/account scripts. Source and released helpers were compared for the supported LBPPC revision. No game data, license keys, extracted executables, account credentials or private Discord material are included in this research. Availability may change after the date above; links lead to each project's own source or announcement.
