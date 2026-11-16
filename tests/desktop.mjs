import { chromium } from '@playwright/test';
import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
const browser=await chromium.connectOverCDP('http://127.0.0.1:9223');
const context=browser.contexts()[0];
const page=context.pages().find(p=>!p.url().startsWith('devtools:'));
await page.waitForSelector('.pod-hero');
const invoke=(command,args={})=>page.evaluate(({command,args})=>window.__TAURI_INTERNALS__.invoke(command,args),{command,args});
await fs.mkdir('docs/screenshots',{recursive:true});
let checks=[];
const check=(name,value)=>{assert.ok(value,name);checks.push(name)};
check('All three games exist',(await invoke('snapshot')).config.games.length===3);
for(const [width,height]of [[1280,720],[1920,1080],[2560,1440]]){
 await page.setViewportSize({width,height});
 await page.screenshot({path:`docs/screenshots/pod-day-${width}.png`,fullPage:true});
 check(`No horizontal overflow at ${width}`,await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth));
}
await page.setViewportSize({width:1280,height:720});
for(const label of ['Library','Downloads','Game files','Settings','About','My Pod']){
 await page.getByRole('button',{name:label,exact:true}).click();
 await page.screenshot({path:`docs/screenshots/${label.toLowerCase().replaceAll(' ','-')}.png`,fullPage:true});
 check(`Navigation ${label}`,await page.locator('main').isVisible());
}
await page.getByRole('button',{name:'Open LittleBigPlanet 3',exact:true}).click();
check('LBP3 native details',await page.getByText('How your world runs').isVisible());
await page.screenshot({path:'docs/screenshots/lbp3.png',fullPage:true});
await page.getByRole('button',{name:'My Pod',exact:true}).click();
await page.getByRole('button',{name:'Use nighttime theme'}).click();
await page.waitForFunction(()=>document.documentElement.dataset.theme==='night');
await page.screenshot({path:'docs/screenshots/pod-night.png',fullPage:true});
check('Theme persisted',(await invoke('snapshot')).config.theme==='night');
await invoke('preferences',{changes:{textScale:150,reducedMotion:true}});
await page.reload();await page.waitForSelector('.pod-hero');
check('150 percent text has no horizontal overflow',await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth));
await page.screenshot({path:'docs/screenshots/text-150.png',fullPage:true});
await invoke('preferences',{changes:{textScale:100,reducedMotion:false,theme:'day'}});
await page.reload();await page.waitForSelector('.pod-hero');
await assert.rejects(()=>invoke('import_data',{id:'lbp1',path:'Z:\\missing-fixture',update:false}));checks.push('Missing import rejected');
await assert.rejects(()=>invoke('check_release',{engine:'replanet'}));checks.push('Unpublished native release is honest');
check('Live LBPPC requirements',(await invoke('refresh_requirements')).version==='1.28');
for(const engine of ['lbppc','rpcs3']){
 const release=await invoke('check_release',{engine});check(`${engine} official release`,release.asset.browser_download_url.startsWith('https://github.com/'));
 await invoke('install',{engine});
 const started=Date.now();let task;
 do{await new Promise(r=>setTimeout(r,1000));task=(await invoke('snapshot')).tasks.find(t=>t.engine===engine);if(Date.now()-started>300000)throw Error('Install timeout');}while(!['Complete','Failed','Cancelled'].includes(task.status));
 check(`${engine} download and install: ${task.error}`,task.status==='Complete');
 check(`${engine} verifies SHA-256`,(await invoke('maintenance',{engine,action:'verify',target:null})).includes('All installed release files match'));
}
await fs.writeFile('docs/desktop-test-results.json',JSON.stringify({checks,count:checks.length,date:new Date().toISOString()},null,2));
console.log(JSON.stringify({passed:checks.length,checks},null,2));
await browser.close();
