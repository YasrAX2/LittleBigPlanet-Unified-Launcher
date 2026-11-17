export const palettes = [
 {id:'cool-island',name:'Cool Island',top:'#247f93',bottom:'#265c47'},
 {id:'ocean',name:'Ocean',top:'#246aa7',bottom:'#173b69'},
 {id:'grape',name:'Grape',top:'#7958a3',bottom:'#3d325e'},
 {id:'berry',name:'Berry',top:'#a04b73',bottom:'#5b2944'},
 {id:'sunshine',name:'Sunshine',top:'#9d7821',bottom:'#55491c'},
 {id:'sand',name:'Sand',top:'#8b7558',bottom:'#453d32'},
 {id:'mint',name:'Mint',top:'#2d876d',bottom:'#184f49'},
 {id:'midnight',name:'Midnight',top:'#334761',bottom:'#15243d'},
] as const;
export const paletteById=(id:string)=>palettes.find(p=>p.id===id)??palettes[0];
