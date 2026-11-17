# Security review - 17 November 2026

Scope: the Windows launcher source, current dependency locks, public assets, reachable Git blobs and GitHub repository protection settings. This is not a penetration-test certification or a promise that the app is safe in every environment.

## Changes made

- ZIP upgraded from 4.6.1 to 9.6.1 after reviewing upstream/RustSec advisories, including https://github.com/zip-rs/zip2/security/advisories/GHSA-m665-v7pm-7grj .
- Abandoned sevenz-rust 0.6.1 replaced with maintained sevenz-rust2 0.25.0, as recommended by https://rustsec.org/advisories/RUSTSEC-2026-0245.html . The custom path, duplicate and link checks remain. Decoder/header memory is capped at 256 MiB per allocation category with one decode thread; actual extracted output is limited to 8 GiB. These are separate caps, not a total process-memory guarantee.
- Abandoned YAML parsing dependencies replaced with YAML Organization's yaml_serde 0.10.7 and libyaml-rs.
- Automatic installation now refuses archives without a usable official SHA-256 checksum, and refuses checksum mismatches. Official HTTPS sources remain fixed in Rust. Checksums verify integrity, not the publisher's trustworthiness or absence of malware.
- Imported artwork is limited to named local PNG/JPEG files, 8 MB each. Traversal filenames and oversized files are rejected. It is never uploaded by the application.
- Public game artwork and screenshots containing it removed from the current source tree. Licensed fonts and the independently drawn launcher icon remain. Local artwork is outside the code repository and release packages.
- CI actions pinned to reviewed commit IDs. Dependency checks and npm audit added to the Windows workflow. Dependabot security updates and private vulnerability reporting enabled. GitHub secret scanning and push protection verified enabled.

## Results

- npm audit: zero reported advisories, including development dependencies.
- OSV: 295 registry packages in the locked Windows graph checked; no matching advisories. See scripts/check_dependencies.py and https://google.github.io/osv.dev/post-v1-querybatch/ .
- The complete cross-platform lockfile still includes six packages with advisories or maintenance notices that are not in the Windows graph (including GTK/glib and Linux portal dependencies). This does not clear other platforms for release.
- Reachable history scan: 336 blobs inspected for supported credential patterns and hardcoded private profile paths; no matches. GitHub secret scanning: zero alerts. This does not prove absence of every possible secret. Commit emails use GitHub noreply addresses.
- Six frontend tests and 24 Rust tests pass, covering settings migration/persistence, invalid blur/artwork values, local file boundaries, checksum enforcement, real byte limits, valid ZIP/7z extraction, traversal rejection and game-file preservation.
- Final native visual verification remains incomplete because the user stopped desktop control. Earlier UI evidence must not be presented as verification of this final revision.

## Remaining limits

The app is unsigned. Upstream engines, imported executables and WebView2 have their own security boundaries. No antivirus verdict or gameplay certification is claimed. Older releases and historical commits still contain previous artwork and older dependency versions; removing current files does not recall those copies. Public-history/release cleanup requires a separate destructive update.
