# Image sources

## Current version 1.2.1

The user supplied public/backgrounds/sackboy.png. It is bundled unchanged (SHA-256 recorded in the package verification), with blur/dimming applied only by CSS. The optional Theme color mode uses CSS colors derived from the selected Popit theme. No AI imagery is used. The forest image is removed from the application; prior credits below are historical. The supplied Sackboy picture remains copyrighted game artwork outside the MIT code license.

## Version 1.2 cover art

The current game selectors and detail pages use covers, not gameplay screenshots:

| File | Source |
| --- | --- |
| public/covers/lbp1.jpg | LittleBigPlanet PS3 front cover from Power Up Gaming: https://www.powerupgaming.ca/products/ps3-little-big-planet |
| public/covers/lbp2.webp | LittleBigPlanet 2 front cover from MobyGames: https://www.mobygames.com/game/52419/littlebigplanet-2/ ; image https://cdn.mobygames.com/4421db9e-ab9e-11ed-8bb9-02420a000197.webp |
| public/covers/lbp3.jpg | LittleBigPlanet 3 PS4 cover from Vandal: https://vandal.elespanol.com/juegos/ps4/littlebigplanet-3/24750 |

These are copyrighted cover/packaging images identifying their respective games; they are excluded from the MIT code license. No open-content license is claimed. The current backdrop is the real forest photograph below, not a game screenshot. Rubik is from Google Fonts under the SIL Open Font License; its full notice accompanies the application.

## Version 1.1 image record

The 1.1 interface uses real game screenshots found through image search. No image-generation service was used. These images are credited to their respective creators and rights holders; they are not original launcher artwork and are not covered by this repository's MIT license.

| Local file | Content | Source |
| --- | --- | --- |
| public/screenshots/lbp1.jpg | LittleBigPlanet, Skate to Victory ghost level | PlayStation.Blog's official Flickr account: https://www.flickr.com/photos/playstationblog/2984668791/ |
| public/screenshots/lbp2.jpg | LittleBigPlanet 2 gameplay | 4Gamer's December 2010 screenshot gallery: https://www.4gamer.net/games/114/G011413/20101220038/screenshot.html |
| public/screenshots/lbp3.jpg | LittleBigPlanet 3 E3 gameplay screenshot | Sony promotional screenshot reproduced by Ars Technica: https://arstechnica.com/gaming/2014/06/sony-e3-roundup-a-smattering-of-uninspired-exclusives/ |
| public/screenshots/pod.png | LittleBigPlanet Pod screenshot | RomStation gallery: https://www.romstation.fr/gallery/album/66432-littlebigplanet-playstation-3 |

LittleBigPlanet game visuals and trademarks belong to Sony Interactive Entertainment and the respective developers. Public availability and attribution do not establish an open-content license. The official Flickr image is marked all rights reserved. The screenshots identify the supported games in this free, unofficial fan launcher. They must not be represented as MIT-licensed assets or as evidence that the launcher played the games during testing. User-imported artwork stays local.

## Current cropped assets and color reference

- public/covers/lbp1-art.webp: cropped lbp1.jpg at (127,62)-(861,929).
- public/covers/lbp2-art.webp: cropped lbp2.webp at (0,60)-(386,440).
- public/covers/lbp3-art.webp: cropped lbp3.jpg at (139,290)-(1171,1396).
- Crops remove platform headers, case/spine and retail margins. The unmodified source files are retained above for attribution.
- public/forest.jpg: real forest photograph from https://images.unsplash.com/photo-1441974231531-c6227db76b6e?auto=format&fit=crop&w=2560&q=85 , under https://unsplash.com/license .
- public/colors/*.png: extracted from the supplied Popit screenshot, preserved as docs/references/player-colors.png. Paired colors are median RGB samples from nine-pixel squares, recorded in color-samples.json. The black/green default reuses the extracted silhouette. The black and mint backing shapes reuse an unobscured backing mask from the same screenshot because the label/background obscured their edges. No image generation was used.

Game icons remain copyrighted game visuals, outside the MIT code license. Colors match the supplied screenshot; compression can differ from original game textures. All current visuals are bundled locally; display makes no image-service requests.
