# Background choices and creator credit

Version 1.2.1 removes the forest and offers exactly two saved choices in Settings:

- Local picture: Choose picture imports a PNG/JPEG from the user's PC. Background blur adjusts from 0 to 24 pixels and persists. The setting is unavailable until a picture is selected.
- Theme color: the default for public builds. It follows the selected Popit color, with a darker shade at night.

Created by yasrax2 appears in the footer, About page, README and package notices. Your private artwork can remain in local application data. Public builds contain no game covers, Sackboy pictures or game icon silhouettes. Game artwork can be imported locally, and is not uploaded by the app. Color choices use original CSS swatches when no local icons exist.

Configuration upgrades preserve engine/game paths. Validation rejects unknown background modes, out-of-range blur and artwork filenames that escape the local art folder. Imported artwork has an 8 MB limit and must have a PNG/JPEG signature.

Desktop visual verification of this final revision remains incomplete because desktop control was stopped. Unit tests and build checks are recorded separately in SECURITY-REVIEW-1.2.1.md.
