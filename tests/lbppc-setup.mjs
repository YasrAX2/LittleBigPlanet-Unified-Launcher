import {chromium} from '@playwright/test';
import {readFileSync,mkdirSync,existsSync,writeFileSync} from 'node:fs';
import {createServer} from 'node:http';
import {join,extname} from 'node:path';
import assert from 'node:assert/strict';
const repo=new URL('../',import.meta.url);
const art=new URL('src-tauri/assets/default-art/',repo);
const uri=name=>'data:image/png;base64,'+readFileSync(new URL(name+'.png',art)).toString('base64');
const palettes=Object.keys(JSON.parse(readFileSync(new URL('manifest.json',art),'utf8'))).filter(n=>n.startsWith('palette-')).map(n=>n.slice(8,-4));
const snapshot={config:{schema:1,theme:'day',palette:'black-green',background:'picture',backgroundBlur:8,backgroundArtwork:'',panelOpacity:55,reducedMotion:false,textScale:100,typography:'current',controller:true,onboarding:false,installRoot:'Choose an engine folder',games:[1,2,3].map(n=>({id:`lbp${n}`,engine:n===3?'lbppc':'rpcs3',data:'',update:'',titleId:'',region:'',version:'',favorite:false,seconds:0,lastPlayed:0,fullscreen:false,artwork:''})),installations:{},activity:[]},tasks:[],running:[],requirements:{titleIds:[],version:'',checked:0,live:false,source:''},backgroundArtwork:uri('background'),paletteArtwork:Object.fromEntries(palettes.map(n=>[n,uri('palette-'+n)])),artwork:Object.fromEntries([1,2,3].map(n=>[`lbp${n}`,uri('lbp'+n)]))};
const dist=new URL('dist/',repo);
const server=createServer((request,response)=>{
 const pathname=decodeURIComponent(new URL(request.url,'http://localhost').pathname);
 const path=join(dist.pathname.replace(/^\/([A-Za-z]:)/,'$1'),pathname==='/'?'index.html':pathname);
 if(!existsSync(path)){response.writeHead(404);response.end();return;}
 const mime={'.html':'text/html','.js':'text/javascript','.css':'text/css','.ttf':'font/ttf','.svg':'image/svg+xml'}[extname(path)]??'application/octet-stream';
 response.writeHead(200,{'Content-Type':mime});response.end(readFileSync(path));
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const browser=await chromium.launch({headless:true,executablePath:'C:/Program Files/Google/Chrome/Application/chrome.exe'});
try{
 const page=await browser.newPage({viewport:{width:1280,height:900}});
 await page.addInitScript(snapshot=>{
  snapshot.config.installations.lbppc={engine:'lbppc',version:'v0.1.5',root:'test',executable:'test',managed:false,rollback:''};
  window.podCalls=[];
  let pending;let signins=0;
  window.__TAURI_INTERNALS__={transformCallback:()=>1,unregisterCallback:()=>{},invoke:async(command,args)=>{
   window.podCalls.push({command,args});
   if(command==='snapshot')return snapshot;
   if(command==='check_release'||command==='refresh_requirements')throw new Error('offline fixture');
   if(command==='lbppc_setup'){
    if(args.action==='status')return {signedIn:false,username:'',online:false};
    if(args.action==='check')return {complete:false,message:'Missing test game files'};
    if(args.action==='signin'){if(++signins>1)return {signedIn:true,username:'Fixture user',online:true};return await new Promise(resolve=>{pending=resolve});}
    return {signedIn:false,username:'',online:false};
   }
   if(command==='cancel_lbppc_signin'){pending({signedIn:false,username:'',online:false});return;}
   return null;
  }};
 },snapshot);
 const errors=[];page.on('pageerror',e=>errors.push(String(e)));
 await page.goto(`http://127.0.0.1:${server.address().port}`);
 await page.locator('.desktop-nav').getByRole('button',{name:'Downloads',exact:true}).click();
 assert.equal(await page.getByRole('button',{name:'Build LBPPC from source',exact:true}).count(),1);
 await page.getByRole('button',{name:'Check and set up game',exact:true}).click();
 await page.getByRole('alert').filter({hasText:'Missing test game files'}).waitFor();
 await page.getByRole('button',{name:'Sign in / Create account',exact:true}).click();
 await page.getByRole('button',{name:'Cancel sign-in',exact:true}).waitFor();
 assert.equal(await page.getByRole('button',{name:'Sign in / Create account',exact:true}).isDisabled(),true);
 await page.getByRole('button',{name:'Cancel sign-in',exact:true}).click();
 await page.getByRole('status').filter({hasText:'Signed out.'}).waitFor();
 await page.getByRole('button',{name:'Sign in / Create account',exact:true}).click();
 await page.getByText('Saved account: Fixture user.',{exact:true}).waitFor();
 await page.getByRole('button',{name:'Sign out',exact:true}).click();
 await page.getByRole('button',{name:'Sign in / Create account',exact:true}).waitFor();
 await page.getByRole('button',{name:'Build LBPPC from source',exact:true}).click();
 const dialog=page.getByRole('dialog').filter({hasText:'Build LBPPC from source?'});
 assert.ok((await dialog.innerText()).includes('20 GB'));
 await dialog.getByRole('button',{name:'Build from source',exact:true}).click();
 await page.waitForFunction(()=>window.podCalls.some(c=>c.command==='build_lbppc'));
 const calls=await page.evaluate(()=>window.podCalls);
 assert.equal(calls.filter(c=>c.command==='build_lbppc').length,1);
 assert.equal(calls.find(c=>c.command==='build_lbppc').args.folder,null);
 assert.deepEqual(errors,[]);
 console.log('LBPPC production UI: one build control, setup error, browser flow, cancellation and confirmed source-build request passed. Native IPC not exercised.');
}finally{await browser.close();server.close();}
