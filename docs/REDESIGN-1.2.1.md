# Background choices

Version 1.2.1 removes the forest and adds exactly two saved background choices in Settings:

- Sackboy picture: the supplied image, bundled unchanged, displayed with 8px CSS blur and a readability dimmer. This is the default.
- Theme color: an image-free background derived from the selected Popit color. It changes immediately when the theme changes, with a darker shade in night mode.

Native radio controls support keyboard selection. The setting persists with other launcher preferences. Older configurations receive the picture default. Invalid background values are rejected before saving. Game paths and saves are unaffected.

No image generation or additional background download is used. See IMAGE-SOURCES.md for the supplied image's provenance.
