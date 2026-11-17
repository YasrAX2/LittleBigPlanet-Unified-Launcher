<p align="center"><img src="public/icon.svg" width="88" alt="Patchwork Pod icon"></p>
<h1 align="center">Patchwork Pod</h1>
<p align="center">Your LittleBigPlanet library, together on Windows.<br>Created by <strong>yasrax2</strong></p>
<p align="center"><a href="https://github.com/YasrAX2/LittleBigPlanet-Unified-Launcher/releases/latest">Download stable</a> · <a href="https://github.com/YasrAX2/LittleBigPlanet-Unified-Launcher/releases/tag/v1.5.0">Try 1.5.0</a> · <a href="docs/USAGE.md">Complete guide</a> · <a href="docs/FAQ.md">Questions & answers</a></p>

Free, unofficial Windows launcher for LittleBigPlanet 1, 2 and 3. Link your games, install their engines and customize your library with translucent Popit-style menus, colors, local covers and backgrounds.

## Start here

1. Download the **x64-setup.exe**, or extract **Windows-x64.zip** and open `patchwork-pod.exe`. The Source ZIP is for developers.
2. Open a game in **Library** and install its engine: RPCS3 for LBP1/2, LBPPC for LBP3.
3. Choose **Game files**, select the game, then **Choose game folder**. Version 1.5.0 also offers **Install PS3 PKG** for LBP1/2.
4. For LBP3, link a matching **1.28 update** unless the game folder already contains that version.
5. **Validate**, then **Play**.

Windows 10/11 x64 and WebView2 are required. Builds are unsigned; SHA256SUMS accompany downloads. No games, firmware, keys or game artwork are bundled.

## Which files do I import?

| Game | Choose | What the folder contains |
| --- | --- | --- |
| LBP1 / LBP2 | Extracted PS3 game folder, or **Install PS3 PKG** in 1.5.0 | `PS3_GAME/PARAM.SFO` and `PS3_GAME/USRDIR/EBOOT.BIN`, or `PARAM.SFO` and `USRDIR/EBOOT.BIN` directly. |
| LBP3 | Extracted **PS4** game folder | `eboot.bin`, `base_001.farc`, `sce_sys/param.sfo`. US CUSA00473 / Europe CUSA00063, update 1.28. |

**A PKG is a file, not a game folder.** The new PS3 PKG button installs it through RPCS3 and links a matching installed LBP folder. Required licenses must come from your own copy. PS4 PKGs are not supported. Extract ZIP/7z/RAR archives before linking their game folders; a lone BIN, FARC or SFO is incomplete. [Folder examples and instructions](docs/USAGE.md#game-imports).

## What you get

- **One library:** Favorites, recent games, local playtime, wide navigation and a compact hamburger menu.
- **Your Popit:** black/green default, nine optional color pairs, panel opacity, theme-color or local-picture backgrounds and 0–24px blur.
- **Readable controls:** text scaling, reduced motion, keyboard/controller navigation and Current / LBP Wiki typography presets.
- **Engine maintenance:** official downloads, verification, repair, move, rollback and retained uninstall backups for managed engines.
- **PS3 setup in 1.5.0:** verified official firmware installation, PS3 package import, automatic linking and an LBP starting profile. Hardware, required licenses and game compatibility still matter.

## FPS support

| Mode | Meaning |
| --- | --- |
| LBPPC Standard | 30 native FPS or 60 displayed using supported frame generation. |
| LBPPC native rendering tests | Opt-in 60/120 render targets, generation off, fixed to official LBPPC 0.1.5, offline with separate saves. |
| RPCS3 | 30/60/120 output caps. Caps do not unlock a game; compatible patches and hardware are needed for higher actual rates. |

**Experimental profiles retain 30 Hz physics.** They do not implement 60/120 Hz simulation or world-motion interpolation. FPS and correct game speed remain unverified in real gameplay. [Video guide](docs/VIDEO.md) · [Test instructions](docs/NATIVE-TESTING.md).

## Help and documentation

| Need | Guide |
| --- | --- |
| Installation, imports, every launcher page, settings and saves | [Complete guide](docs/USAGE.md) |
| “Can I use a PKG?”, firmware, licenses and other common questions | [Q&A](docs/FAQ.md) |
| Missing files, wrong regions, failed downloads or startup | [Troubleshooting](docs/TROUBLESHOOTING.md) |
| Report a problem | [Issues](https://github.com/YasrAX2/LittleBigPlanet-Unified-Launcher/issues) |
| Privacy, security and verification | [Privacy](PRIVACY.md) · [Security](SECURITY.md) · [Testing](docs/TESTING.md) |

<details>
<summary><strong>Build from source</strong></summary>

Node.js 24+, Rust 1.93+ MSVC, Visual Studio C++ build tools, Windows SDK and WebView2 on Windows x64.

```powershell
npm ci
python -m unittest discover -s tests -p "test_*.py"
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml --locked
npm run tauri -- build --bundles nsis
```

Development: `npm run tauri -- dev`. The frontend requires the Tauri backend. [Upstream research](docs/UPSTREAM.md).

</details>

## Credits

Created by **yasrax2**. Thanks to [LBPPC](https://github.com/emurphy04/LBPPC), [RPCS3](https://github.com/RPCS3/rpcs3) and the libraries listed in [third-party notices](THIRD-PARTY-NOTICES.md). Launcher source: [MIT](LICENSE).

No launcher account, ads or analytics. Imported pictures stay local; automatic release checks contact GitHub. Unofficial fan project, not affiliated with or endorsed by Sony Interactive Entertainment, Media Molecule or Sumo Digital. Game names and artwork belong to their respective owners. [Artwork and distribution](docs/LEGAL.md).
