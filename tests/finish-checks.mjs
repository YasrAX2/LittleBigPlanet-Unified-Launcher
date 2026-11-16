import {chromium} from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
const b=await chromium.connectOverCDP('http://127.0.0.1:9223');const p=b.contexts()[0].pages()[0];
const invoke=(command,args={})=>p.evaluate(({command,args})=>window.__TAURI_INTERNALS__.invoke(command,args),{command,args});
let results=[];
for(const theme of ['day','night']){
 await invoke('preferences',{changes:{theme,textScale:100}});await p.reload();await p.waitForSelector('.pod-hero');
 await p.screenshot({path:`docs/screenshots/pod-${theme}-1280.png`,fullPage:true});
 const axe=await new AxeBuilder({page:p}).withTags(['wcag2a','wcag2aa','wcag21aa']).analyze();
 results.push({theme,violations:axe.violations.map(v=>({id:v.id,impact:v.impact,nodes:v.nodes.map(n=>({target:n.target,summary:n.failureSummary}))}))});
}
await p.getByRole('button',{name:'Downloads',exact:true}).click();await p.screenshot({path:'docs/screenshots/downloads-complete.png',fullPage:true});
for(const engine of ['lbppc','rpcs3']){
 const v=await invoke('maintenance',{engine,action:'verify',target:null});assert.ok(v.includes('All installed release files match'));
 await invoke('install',{engine});const start=Date.now();let task;
 do{await new Promise(r=>setTimeout(r,1000));task=(await invoke('snapshot')).tasks.find(t=>t.engine===engine);if(Date.now()-start>300000)throw Error('Repair timeout')}while(!['Complete','Failed','Cancelled'].includes(task.status));
 assert.equal(task.status,'Complete',task.error);await invoke('maintenance',{engine,action:'rollback',target:null});
 results.push({engine,repair:'passed',rollback:'passed'});
}
await invoke('preferences',{changes:{theme:'day'}});
await fs.writeFile('docs/extended-test-results.json',JSON.stringify(results,null,2));console.log(JSON.stringify(results,null,2));await b.close();
