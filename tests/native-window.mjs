import {chromium} from '@playwright/test';
import {execFileSync} from 'node:child_process';
import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
const browser=await chromium.connectOverCDP(process.env.POD_CDP??'http://127.0.0.1:9223');
const page=browser.contexts()[0].pages()[0];
const cdp=await page.context().newCDPSession(page);
await cdp.send('Emulation.clearDeviceMetricsOverride');
const rows=[];
for(const mode of['compact','maximize','compact','restore']){
 const maximized=execFileSync('powershell.exe',['-NoProfile','-File','tests/resize-window.ps1','-AppProcess',process.env.POD_PROCESS,'-Mode',mode],{encoding:'utf8',windowsHide:true}).trim();
 await page.waitForTimeout(500);
 const state=await page.evaluate(()=>({width:innerWidth,height:innerHeight,menu:getComputedStyle(document.querySelector('.menu-toggle')).display!=='none',navigation:getComputedStyle(document.querySelector('.desktop-nav')).display!=='none',modal:document.querySelector('#navigation-menu').open}));
 assert.equal(state.menu,state.width<1200);assert.equal(state.navigation,state.width>=1200);
 if(mode==='maximize'){assert.equal(maximized,'True');assert.equal(state.menu,false);assert.equal(state.modal,false)}
 if(mode==='compact'){assert.equal(state.menu,true);await page.locator('.menu-toggle').click()}
 if(mode==='restore'){assert.equal(state.modal,false);assert.equal(state.menu,false)}
 rows.push({mode,maximized,state});
}
await fs.writeFile('docs/native-window-results.json',JSON.stringify(rows,null,2)+'\n');
console.log(JSON.stringify(rows));await browser.close();
