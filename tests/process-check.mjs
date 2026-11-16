import {chromium} from '@playwright/test';
import fs from 'node:fs/promises';import path from 'node:path';import assert from 'node:assert/strict';
const root=path.resolve('../fixtures');const game=path.join(root,'game');await fs.mkdir(path.join(game,'USRDIR'),{recursive:true});
let keys=[],values=[],entries=[];
for(const [key,value]of [['TITLE','LittleBigPlanet'],['TITLE_ID','BCES00141'],['APP_VER','02.00']]){let e=Buffer.alloc(16);e.writeUInt16LE(keys.length,0);e.writeUInt16LE(0x204,2);e.writeUInt32LE(value.length+1,4);e.writeUInt32LE(value.length+1,8);e.writeUInt32LE(values.length,12);entries.push(e);keys.push(...Buffer.from(key),0);values.push(...Buffer.from(value),0)}
let header=Buffer.alloc(20);header.write('\0PSF');header.writeUInt32LE(0x101,4);header.writeUInt32LE(68,8);header.writeUInt32LE(68+keys.length,12);header.writeUInt32LE(3,16);await fs.writeFile(path.join(game,'PARAM.SFO'),Buffer.concat([header,...entries,Buffer.from(keys),Buffer.from(values)]));await fs.writeFile(path.join(game,'USRDIR/EBOOT.BIN'),'synthetic fixture only');
const b=await chromium.connectOverCDP('http://127.0.0.1:9223');const p=b.contexts()[0].pages()[0];const invoke=(command,args={})=>p.evaluate(({command,args})=>window.__TAURI_INTERNALS__.invoke(command,args),{command,args});
await invoke('import_engine',{engine:'lbprecomp',path:path.join(root,'engine/fixture.exe')});await invoke('import_data',{id:'lbp1',path:game,update:false});await invoke('game_settings',{id:'lbp1',changes:{engine:'lbprecomp'}});await invoke('launch',{id:'lbp1',settings:false});
assert.ok((await invoke('snapshot')).running.includes('lbp1'));await new Promise(r=>setTimeout(r,4000));let snap=await invoke('snapshot');assert.ok(!snap.running.includes('lbp1'));assert.ok(snap.config.games[0].seconds>=1);
await invoke('game_settings',{id:'lbp1',changes:{engine:'rpcs3'}});await invoke('clear_data',{id:'lbp1',update:false});await invoke('maintenance',{engine:'lbprecomp',action:'forget',target:null});
assert.equal(await fs.readFile(path.join(game,'USRDIR/EBOOT.BIN'),'utf8'),'synthetic fixture only');
await fs.writeFile('docs/process-test-results.json',JSON.stringify({syntheticImport:'passed',externalProcess:'passed',playtime:'passed',originalFilesUnchanged:'passed',realGamePlayed:false},null,2));console.log('Synthetic import, real process launch, playtime and original-file preservation passed.');await b.close();
