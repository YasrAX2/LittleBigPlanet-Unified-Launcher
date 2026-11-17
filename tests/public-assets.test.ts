import {it,expect} from 'vitest';
import {readdirSync} from 'node:fs';
it('ships only fonts and the original launcher icon in web assets',()=>{
 const files=readdirSync(new URL('../public/',import.meta.url),{recursive:true,withFileTypes:true}).filter(f=>f.isFile());
 expect(files.length).toBeGreaterThan(0);
 for(const file of files)expect(file.name==='icon.svg'||/\.(ttf|txt)$/i.test(file.name)).toBe(true);
});
