# Popit appearance and navigation

Version 1.2 follows the supplied Popit references:

- Black glass with green accents is the default. Nine optional color pairs use RGB samples and Sackboy silhouettes extracted from the supplied reference, in the same three-by-three order. The reference and samples are under references/. No AI imagery is used.
- Below 1200 CSS pixels, a hamburger opens a modal navigation drawer. At 1200 and above, navigation stays visible and the hamburger disappears. Expanding closes an open drawer and its backdrop. Escape, focus handling and controller navigation remain supported. See the window-width principle in https://developer.android.com/develop/ui/views/layout/build-responsive-navigation .
- Covers fill their outlines. Retail platform headers and packaging margins are cropped out. A real Unsplash forest photograph replaces the game screenshot backdrop. Assets are bundled for offline display; provenance is in IMAGE-SOURCES.md.
- Translucent panels retain blur and a saved opacity control. Light color pairs use dark text and a light backing for contrast. The black default uses the selected opacity directly. Reduced motion is respected separately.
- Rubik matches the supplied wiki typography reference. Fandom's desktop font is documented as Rubik: https://community.fandom.com/f/p/4400000000003210774 . The screenshot was used directly; the Popit article refused automated fetching.

Legacy configurations receive black/green and 65% defaults. Superseded draft palettes migrate to black/green without changing game paths, installations or saves. Configuration restore uses the same migration. Unsupported values are rejected before saving.

Verification: reference-popit-results.json and screenshots-v1.2/. Desktop WebView checks cover six widths, the breakpoint boundary, drawer expansion/Escape, ten choices, exact color tokens, image loading, persistence, all six pages in both display modes, game detail pages and 150% text. The 24 automated accessibility scans found no violations; they are not a complete accessibility audit. Native maximize/restore behavior is recorded in native-window-results.json. Gameplay and physical controllers remain untested in this version.
