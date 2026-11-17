import {chromium} from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
const browser=await chromium.connectOverCDP(process.env.POD_CDP??'http://127.0.0.1:9223');
const page=browser.contexts()[0].pages()[0];
const invoke=(command,args={})=>page.evaluate(({command,args})=>window.__TAURI_INTERNALS__.invoke(command,args),{command,args});
const results=[];
const directory='docs/screenshots-v1.2';
await fs.mkdir(directory,{recursive:true});
await page.reload();await page.waitForSelector('.world-bubbles');
await invoke('preferences',{changes:{palette:'black-green',panelOpacity:65,textScale:100,theme:'day'}});
await page.reload();await page.waitForSelector('.world-bubbles');
async function navigate(label){
 if(await page.locator('.menu-toggle').isVisible()){
  await page.locator('.menu-toggle').click();
  await page.locator('#navigation-menu nav').getByRole('button',{name:label,exact:true}).click();
  assert.equal(await page.locator('#navigation-menu').isVisible(),false);
 }else await page.locator('.desktop-nav').getByRole('button',{name:label,exact:true}).click();
}
async function scan(label){
 const scan=await new AxeBuilder({page}).withTags(['wcag2a','wcag2aa','wcag21aa']).analyze();
 results.push({check:label,violations:scan.violations.map(v=>({id:v.id,nodes:v.nodes.map(n=>({target:n.target,summary:n.failureSummary}))}))});
}
for(const [width,height]of[[960,640],[1199,800],[1200,800],[1280,720],[1920,1080],[2560,1440]]){
 await page.setViewportSize({width,height});
 assert.equal(await page.locator('.menu-toggle').isVisible(),width<1200);
 assert.equal(await page.locator('.desktop-nav').isVisible(),width>=1200);
 assert.ok(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth));
 for(const img of await page.locator('.world-bubbles img').all()){
  assert.ok(await img.evaluate(e=>e.complete&&e.naturalWidth>100));
  assert.match(await img.getAttribute('src'),/-art\.webp$/);
  assert.equal(await img.evaluate(e=>getComputedStyle(e).objectFit),'cover');
 }
 await page.screenshot({path:`${directory}/home-${width}.png`,fullPage:true});
 results.push({check:`Responsive navigation and edge-filled covers ${width}x${height}`,passed:true});
}
await page.setViewportSize({width:960,height:640});
await page.locator('.menu-toggle').click();await page.keyboard.press('Escape');assert.equal(await page.locator('#navigation-menu').isVisible(),false);
await page.locator('.menu-toggle').click();await page.setViewportSize({width:1280,height:800});await page.waitForTimeout(100);
assert.equal(await page.locator('#navigation-menu').isVisible(),false);assert.equal(await page.locator('.menu-toggle').isVisible(),false);
results.push({check:'Escape and expansion close the compact modal without leaving a backdrop',passed:true});
await navigate('Settings');
assert.equal(await page.locator('.palette-choice').count(),10);
assert.ok((await page.locator('.main').evaluate(e=>getComputedStyle(e).backdropFilter)).includes('blur'));
await page.screenshot({path:`${directory}/appearance-black-green.png`,fullPage:true,mask:[page.locator('.path')],maskColor:'#263e33'});
const samples=JSON.parse(await fs.readFile('docs/references/color-samples.json','utf8'));
for(const p of [{id:'black-green',name:'Black & green',top:'#080b0a',bottom:'#080b0a'},...samples]){
 const button=page.getByRole('button',{name:`${p.name} color`,exact:true});
 await button.click();await page.waitForTimeout(100);
 assert.equal(await button.getAttribute('aria-pressed'),'true');
 assert.equal(await page.evaluate(()=>document.documentElement.dataset.palette),p.id);
 const rgb=h=>[1,3,5].map(i=>parseInt(h.slice(i,i+2),16)).join(' ');
 assert.equal(await page.evaluate(()=>document.documentElement.style.getPropertyValue('--glass-top')),rgb(p.top));
 assert.equal(await page.evaluate(()=>document.documentElement.style.getPropertyValue('--glass-bottom')),rgb(p.bottom));
 assert.ok(await button.locator('img').evaluate(e=>e.complete&&e.naturalWidth===84));
 await scan(`${p.name} reference colors and icon`);
}
await page.getByRole('button',{name:'Violet & green color',exact:true}).click();
await page.getByRole('slider',{name:'Panel opacity',exact:true}).fill('75');await page.waitForTimeout(100);
await page.reload();await page.waitForSelector('.world-bubbles');
const snapshot=await invoke('snapshot');assert.equal(snapshot.config.palette,'violet-green');assert.equal(snapshot.config.panelOpacity,75);
await navigate('Settings');await page.screenshot({path:`${directory}/appearance-violet-green.png`,fullPage:true,mask:[page.locator('.path')],maskColor:'#263e33'});
results.push({check:'Reference color and opacity persist after reload',passed:true});
await invoke('preferences',{changes:{palette:'black-green',panelOpacity:65}});
for(const theme of ['day','night']){
 await invoke('preferences',{changes:{theme}});await page.reload();await page.waitForSelector('.world-bubbles');
 for(const label of ['Home','Library','Downloads','Game files','Settings','About']){
  await navigate(label);await scan(`${theme} ${label}`);
  await page.screenshot({path:`${directory}/${theme}-${label.toLowerCase().replaceAll(' ','-')}.png`,fullPage:true,mask:[page.locator('.path')],maskColor:'#263e33'});
 }
}
await navigate('Library');
for(const title of ['LittleBigPlanet','LittleBigPlanet 2','LittleBigPlanet 3']){
 await page.getByRole('button',{name:`Open ${title}`,exact:true}).click();
 assert.ok(await page.getByRole('heading',{name:title,exact:true,level:1}).isVisible());
 assert.match(await page.locator('.game-hero-art img').getAttribute('src'),/-art\.webp$/);
 await page.getByRole('button',{name:'Back to library',exact:true}).click();
}
await invoke('preferences',{changes:{textScale:150}});await page.reload();await page.waitForSelector('.world-bubbles');
for(const width of[960,1280]){await page.setViewportSize({width,height:800});await navigate('Settings');assert.ok(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth));await scan(`Text 150 percent at ${width}`)}
await invoke('preferences',{changes:{textScale:100,theme:'day',palette:'black-green',panelOpacity:65}});
await page.reload();await page.waitForSelector('.world-bubbles');await page.setViewportSize({width:1280,height:800});
assert.equal(await page.locator('.pod-backdrop img').getAttribute('src'),'/forest.jpg');
await fs.writeFile('docs/reference-popit-results.json',JSON.stringify(results,null,2)+'\n');
console.log(JSON.stringify(results.map(r=>({...r,violations:r.violations?.map(v=>({id:v.id,targets:v.nodes.map(n=>n.target)}))})),null,2));
await browser.close();
assert.ok(results.every(r=>!r.violations?.length),'Accessibility issues recorded in docs/reference-popit-results.json');
