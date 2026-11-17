# Patchwork Pod

**LittleBigPlanet Unified Launcher** for Windows. Created by **yasrax2**.

An independent fan launcher built with Rust, Tauri 2, React and TypeScript. Version 1.4.0 is a prerelease adding experimental native-rendering targets and a selectable LBP Wiki typography preset. It retains translucent Popit panels, black/green defaults, nine reference color pairs, local pictures with adjustable blur, local game covers, responsive navigation and licensed Rubik typography. Public builds contain no game artwork, game data, firmware, keys or music.

## Experimental tests

[1.4.0 test release](https://github.com/YasrAX2/LittleBigPlanet-Unified-Launcher/releases/tag/v1.4.0) offers opt-in 60/120 native rendering requests using existing LBPPC research hooks, with frame generation off and separate offline saves. Physics remains 30 Hz. Motion can still step at 30 Hz, and actual FPS/game speed are unverified. This is not stable native 60/120 simulation support. [Test instructions and limits](docs/NATIVE-TESTING.md).

In Settings, choose Current style or LBP Wiki (Rubik with heavier headings). The existing licensed font is reused.

## Download

[Windows installer and portable ZIP](https://github.com/YasrAX2/LittleBigPlanet-Unified-Launcher/releases/latest). Use SHA256SUMS to verify downloads. The application is unsigned. See [usage](docs/USAGE.md), [troubleshooting](docs/TROUBLESHOOTING.md) and [video settings](docs/VIDEO.md).

Background choices: a picture you import locally with 0-24px blur, or a solid color matching your selected theme. Imported artwork stays on your PC.

## Supported engines

| Game | Windows option | Native project |
| --- | --- | --- |
| LittleBigPlanet | RPCS3, extracted PS3 game folders | LittleBigRecomp currently targets Apple Silicon macOS; future official Windows builds can be discovered/imported |
| LittleBigPlanet 2 | RPCS3, extracted PS3 game folders | RePlanet has no verified public release endpoint; future official Windows executables can be imported |
| LittleBigPlanet 3 | Official LBPPC GitHub Windows release | US CUSA00473 and Europe CUSA00063, game update 1.28, checked 17 November 2026 |

LBPPC offers 30 native FPS or 60 displayed FPS with frame generation. Native 120 FPS is not implemented upstream. RPCS3 offers 30/60/120 caps; compatible game patches and sufficient hardware are still needed for higher rates. [Details and sources](docs/VIDEO.md).

LBPPC compatibility requirements refresh from its official README and are cached for offline use. Unexpected formats produce an error instead of silently changing requirements. The engine remains the final authority on compatibility.

## Build

Windows 10/11 x64, Node.js 24+, Rust 1.93+ (MSVC), Visual Studio C++ build tools, Windows SDK and WebView2.

```powershell
npm ci
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml --locked
npm run tauri -- build --bundles nsis
```

Development: `npm run tauri -- dev`. The browser frontend requires the real Tauri backend. It never fabricates installed games or successful actions.

## Privacy and ownership

Game folders are referenced read-only. Engines live separately. Imported engines are never automatically updated or uninstalled. Managed updates retain previous installations and preserve user-created files/settings. Uninstall moves a managed engine into a retained backup instead of deleting saves.

No launcher account, ads or analytics. Automatic release and requirements checks contact GitHub. Diagnostics are generated locally and omit paths, usernames, logs, artwork and credentials. Backups can contain private paths or engine settings; keep them private. Engine online accounts remain separate.

See [privacy notice](PRIVACY.md), [artwork and distribution](docs/LEGAL.md), [image sources](docs/IMAGE-SOURCES.md) and [review of the supplied checklist](docs/CHECKLIST-REVIEW.md).

## Project notes

[Upstream research](docs/UPSTREAM.md), [third-party notices](THIRD-PARTY-NOTICES.md), [security policy](SECURITY.md), [security review](docs/SECURITY-REVIEW-1.2.1.md) and [verification](docs/TESTING.md).

Unofficial fan project, not affiliated with or endorsed by Sony Interactive Entertainment, Media Molecule or Sumo Digital. LittleBigPlanet and related trademarks belong to their owners. The MIT code license grants no rights to their games or artwork.
