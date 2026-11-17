import {it,expect} from 'vitest';
import {readFileSync,readdirSync} from 'node:fs';
import {createHash} from 'node:crypto';
it('bundles exactly the approved default pictures, without local configuration',()=>{
 const dir=new URL('../src-tauri/assets/default-art/',import.meta.url);
 const manifest:Record<string,string>=JSON.parse(readFileSync(new URL('manifest.json',dir),'utf8'));
 const expected=['background.png','lbp1.png','lbp2.png','lbp3.png',...['amber-mint','black-green','blue-grey','cyan-yellow','olive-maroon','peach-lavender','pink-cream','purple-black','red-burgundy','violet-green'].map(id=>`palette-${id}.png`)];
 expect(Object.keys(manifest).sort()).toEqual(expected.sort());
 expect(readdirSync(dir).sort()).toEqual([...expected,'manifest.json'].sort());
 for(const name of expected){
  const bytes=readFileSync(new URL(name,dir));
  expect(bytes.subarray(0,8).toString('hex')).toBe('89504e470d0a1a0a');
  expect(createHash('sha256').update(bytes).digest('hex')).toBe(manifest[name]);
 }
});
