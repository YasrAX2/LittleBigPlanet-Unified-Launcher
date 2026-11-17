import {expect,it} from 'vitest';
import {palettes,paletteById} from './appearance';
it('keeps a deterministic palette for legacy configurations',()=>{
 expect(paletteById('missing').id).toBe('cool-island');
 expect(new Set(palettes.map(p=>p.id)).size).toBe(palettes.length);
 for(const p of palettes){expect(p.top).toMatch(/^#[0-9a-f]{6}$/);expect(p.bottom).toMatch(/^#[0-9a-f]{6}$/)}
});
