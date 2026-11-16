import { invoke } from '@tauri-apps/api/core';
export type Game={id:string;engine:string;data:string;update:string;titleId:string;region:string;version:string;favorite:boolean;seconds:number;lastPlayed:number;fullscreen:boolean;artwork:string};
export type Installation={engine:string;executable:string;version:string;managed:boolean;root:string;rollback:string};
export type Config={schema:number;theme:string;reducedMotion:boolean;textScale:number;controller:boolean;onboarding:boolean;installRoot:string;games:Game[];installations:Record<string,Installation>;activity:string[]};
export type Task={engine:string;version:string;status:string;downloaded:number;total:number;speed:number;error:string};
export type Requirements={titleIds:string[];version:string;checked:number;live:boolean;source:string};
export type Snapshot={config:Config;tasks:Task[];running:string[];requirements:Requirements;artwork:Record<string,string>};
export type Release={engine:string;version:string;page:string;notes:string;asset:{name:string;size:number;digest?:string}};
export type Inspection={path:string;titleId:string;title:string;region:string;version:string;valid:boolean;messages:string[]};
export const call=<T=unknown>(command:string,args:Record<string,unknown>={})=>invoke<T>(command,args);
export const catalog=[
 {id:'lbp1',number:'01',title:'LittleBigPlanet',subtitle:'Where imagination began.',world:'The first adventure',color:'#7f9e53',native:'LittleBigRecomp currently targets Apple Silicon macOS with Metal. Use RPCS3 on Windows.',source:'lbprecomp',badge:'RPCS3'},
 {id:'lbp2',number:'02',title:'LittleBigPlanet 2',subtitle:'A whole new world of possibility.',world:'Dream a little bigger',color:'#cc735b',native:'RePlanet is in development. No verified public Windows release source is configured. Use RPCS3 today.',source:'replanet',badge:'RPCS3'},
 {id:'lbp3',number:'03',title:'LittleBigPlanet 3',subtitle:'Big adventures. Little surprises.',world:'Something wonderful awaits',color:'#6686aa',native:'LBPPC is a public native Windows beta. Its own launcher handles graphics, controls, saves and online accounts.',source:'lbppc',badge:'Native · LBPPC'},
];
export const engineName=(s:string)=>({rpcs3:'RPCS3',lbppc:'LBPPC',lbprecomp:'LittleBigRecomp',replanet:'RePlanet'}[s]??s);
export const bytes=(n:number)=>n>=1073741824?`${(n/1073741824).toFixed(1)} GB`:n>=1048576?`${(n/1048576).toFixed(1)} MB`:`${Math.round(n/1024)} KB`;
export const playtime=(s:number)=>s<60?'Not played yet':s<3600?`${Math.floor(s/60)} min`:`${Math.floor(s/3600)} h ${Math.floor(s%3600/60)} min`;
export const activeTask=(t:Task)=>!['Complete','Cancelled','Failed'].includes(t.status);
export const progress=(t:Task)=>t.total>0?Math.min(100,Math.floor(t.downloaded/t.total*100)):0;
