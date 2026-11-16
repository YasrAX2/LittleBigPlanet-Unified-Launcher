# Popit redesign

The user requested real game screenshots, LBP bubble menus, and removal of generic copy and decoration, using [unslop](https://github.com/theclaymethod/unslop) as a reference.

Version 1.1 replaces the SVG globe and landscape illustrations with real LBP1/2/3 screenshots and a Pod background. Game entries are circular. Six navigation bubbles sit inside a Popit-style menu, and dialogs use the same visual language. Slogans and decorative all-caps labels were replaced with task names such as Game library, Downloads, Game files and Settings. The application icon uses a simple bubble symbol.

The unslop core contract and crisp preset guided copy edits. Phrase and structure scanners reported no hard findings in the replacement sample. Preservation passed with a warning about removing the marketing phrase "Always yours"; the concrete read-only game-data guarantees remain in the setup instructions and unchanged backend. Soft prose-shape and repetition warnings were reviewed as UI-label false positives. These scanners do not evaluate visual design.

Actual Tauri desktop checks passed at 1280x720, 1920x1080 and 2560x1440. Screenshot loading, favorites, filters, all three game pages, saved preferences and 150% text scaling passed. Twelve section/theme accessibility scans report zero violations. Results: popit-test-results.json. Screenshots: screenshots-v1.1/.

Engine, download, filesystem and process logic is unchanged. Previous backend tests and limitations remain applicable. Gameplay imagery is not evidence that the launcher played a commercial game during testing. Current image sources and rights are recorded in IMAGE-SOURCES.md. The screenshots-v1.0 gallery is retained for historical comparison under screenshots/.
