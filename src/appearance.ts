// Color values from the supplied reference; no reference artwork is distributed.
export const palettes = [
 {
  "id": "black-green",
  "name": "Black & green",
  "top": "#080b0a",
  "bottom": "#080b0a",
  "accent": "#40f6a3"
 },
 {
  "id": "red-burgundy",
  "name": "Red & burgundy",
  "top": "#cf1044",
  "bottom": "#570f37"
 },
 {
  "id": "purple-black",
  "name": "Purple & black",
  "top": "#cd00d9",
  "bottom": "#000000"
 },
 {
  "id": "pink-cream",
  "name": "Pink & cream",
  "top": "#f7e4e6",
  "bottom": "#cad88d"
 },
 {
  "id": "blue-grey",
  "name": "Blue & grey",
  "top": "#3091f8",
  "bottom": "#c2baaf"
 },
 {
  "id": "olive-maroon",
  "name": "Olive & maroon",
  "top": "#9e9047",
  "bottom": "#45080f"
 },
 {
  "id": "amber-mint",
  "name": "Amber & mint",
  "top": "#f8ac4c",
  "bottom": "#7edd81"
 },
 {
  "id": "violet-green",
  "name": "Violet & green",
  "top": "#b162cd",
  "bottom": "#40f6a3"
 },
 {
  "id": "peach-lavender",
  "name": "Peach & lavender",
  "top": "#f4a85a",
  "bottom": "#adb3e3"
 },
 {
  "id": "cyan-yellow",
  "name": "Cyan & yellow",
  "top": "#47fffd",
  "bottom": "#dbd908"
 }
] as const;
export const paletteById=(id:string)=>palettes.find(p=>p.id===id)??palettes[0];
