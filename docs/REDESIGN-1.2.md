# Glass Popit redesign

Version 1.2 follows the user's Popit and wiki references:

- Navigation opens from a labeled hamburger button. It starts closed, closes after selection, and supports Escape and native dialog focus handling. The former fixed right-side menu is removed.
- Home and library entries use downloaded game cover art. Full images are contained rather than cut into screenshot circles. Detail pages use the same cover. Custom user artwork still takes precedence.
- Menus, dialogs and content panels use translucent tinted backgrounds and backdrop blur. Eight color presets and a 55–95% opacity slider apply immediately and persist in launcher settings. The explicit launcher opacity setting controls this rendering; reduced motion remains supported separately.
- Rubik replaces the former heading/body fonts. Fandom's default desktop font is documented as Rubik: https://community.fandom.com/f/p/4400000000003210774 . The supplied wiki screenshot was also used as a visual reference; the article itself refused automated fetching.
- Appearance has a direct header shortcut. "My Pod" is renamed "Home" in navigation.

The existing configuration schema remains compatible. Legacy settings receive Cool Island and 65% opacity defaults. Unknown palette names and out-of-range opacity values are rejected without changing saved settings.

Verification records: glass-popit-results.json. Actual desktop captures: screenshots-v1.2/. The checks cover four desktop sizes, menu opening/closing, all eight presets, persisted opacity/color, favorites, game pages and 150% text. Accessibility scans cover all six pages in both display modes and all preset colors. As before, automated scans are not a complete accessibility audit. Gameplay and physical-controller tests remain unavailable.
