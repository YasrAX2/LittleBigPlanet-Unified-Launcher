import {chromium} from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
const browser=await chromium.connectOverCDP('http://127.0.0.1:9223');
const page=browser.contexts()[0].pages()[0];
const invoke=(command,args={})=>page.evaluate(({command,args})=>window.__TAURI_INTERNALS__.invoke(command,args),{command,args});
await fs.mkdir('docs/screenshots-v1.1',{recursive:true});
await page.getByRole('button',{name:'My Pod',exact:true}).click();await page.waitForSelector('.world-bubbles');
let results=[];
for(const [width,height] of [[1280,720],[1920,1080],[2560,1440]]){
 await page.setViewportSize({width,height});
 assert.ok(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth));
 for(const image of await page.locator('.world-bubbles .card-art img').all())assert.ok(await image.evaluate(i=>i.complete&&i.naturalWidth>100));
 await page.screenshot({path:`docs/screenshots-v1.1/pod-${width}.png`,fullPage:true});
 results.push({check:`Real screenshots and layout ${width}x${height}`,passed:true});
}
await page.setViewportSize({width:1280,height:720});
for(const theme of ['day','night']){
 await invoke('preferences',{changes:{theme}});await page.reload();await page.waitForSelector('.world-bubbles');
 for(const label of ['My Pod','Library','Downloads','Game files','Settings','About']){
  await page.getByRole('button',{name:label,exact:true}).click();await page.waitForTimeout(200);
  assert.ok(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth));
  const axe=await new AxeBuilder({page}).withTags(['wcag2a','wcag2aa','wcag21aa']).analyze();
  results.push({theme,page:label,violations:axe.violations.map(v=>({id:v.id,nodes:v.nodes.map(n=>({target:n.target,summary:n.failureSummary}))}))});
  await page.screenshot({path:`docs/screenshots-v1.1/${theme}-${label.toLowerCase().replaceAll(' ','-')}.png`,fullPage:true,mask:[page.locator('.path')],maskColor:'#b9c6ca'});
 }
 await page.getByRole('button',{name:'My Pod',exact:true}).click();
}
await page.getByRole('button',{name:'Favorite LittleBigPlanet',exact:true}).click();
await page.getByRole('button',{name:'Library',exact:true}).click();await page.getByRole('button',{name:'Favorites',exact:true}).click();assert.equal(await page.locator('.game-card').count(),1);
await page.getByRole('button',{name:'Unfavorite LittleBigPlanet',exact:true}).click();assert.equal(await page.locator('.game-card').count(),0);
await page.getByRole('button',{name:'All games',exact:true}).click();
for(const title of ['LittleBigPlanet','LittleBigPlanet 2','LittleBigPlanet 3']){
 await page.getByRole('button',{name:`Open ${title}`,exact:true}).click();assert.ok(await page.getByRole('heading',{name:title,exact:true,level:1}).isVisible());await page.getByRole('button',{name:'Back to your worlds'}).click();
}
results.push({check:'Favorites, filters and all three game pages',passed:true});
await invoke('preferences',{changes:{textScale:150,reducedMotion:true,theme:'day'}});await page.reload();await page.waitForSelector('.world-bubbles');
assert.ok(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth));results.push({check:'150% text size and saved preferences',passed:true});
await invoke('preferences',{changes:{textScale:100,reducedMotion:false}});await page.reload();await page.waitForSelector('.world-bubbles');
await fs.writeFile('docs/popit-test-results.json',JSON.stringify(results,null,2));
console.log(JSON.stringify(results.map(r=>({...r,violations:r.violations?.map(v=>({id:v.id,targets:v.nodes.map(n=>n.target)}))})),null,2));
await browser.close();
