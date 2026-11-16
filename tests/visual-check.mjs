import {chromium} from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import fs from 'node:fs/promises';
const b=await chromium.connectOverCDP('http://127.0.0.1:9223');const p=b.contexts()[0].pages()[0];
let results=[];
for(const theme of ['day','night']){
 await p.evaluate(theme=>window.__TAURI_INTERNALS__.invoke('preferences',{changes:{theme}}),theme);await p.reload();await p.waitForSelector('.pod-hero');
 for(const label of ['My Pod','Library','Downloads','Game files','Settings','About']){
  await p.getByRole('button',{name:label,exact:true}).click();await p.waitForTimeout(450);
  const axe=await new AxeBuilder({page:p}).withTags(['wcag2a','wcag2aa','wcag21aa']).analyze();
  results.push({theme,page:label,violations:axe.violations.map(v=>({id:v.id,nodes:v.nodes.map(n=>({target:n.target,summary:n.failureSummary}))}))});
  await p.screenshot({path:`docs/screenshots/${theme}-${label.toLowerCase().replaceAll(' ','-')}.png`,fullPage:true,mask:[p.locator('.path')],maskColor:'#d5cbb7'});
 }
 await p.getByRole('button',{name:'My Pod',exact:true}).click();
}
await p.evaluate(()=>window.__TAURI_INTERNALS__.invoke('preferences',{changes:{theme:'day'}}));
await fs.writeFile('docs/accessibility-results.json',JSON.stringify(results,null,2));console.log(JSON.stringify(results.map(r=>({theme:r.theme,page:r.page,violations:r.violations.map(v=>({id:v.id,targets:v.nodes.map(n=>n.target)}))})),null,2));await b.close();
