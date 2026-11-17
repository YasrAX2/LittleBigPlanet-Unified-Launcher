import samples from '../docs/references/color-samples.json';
import {expect,it} from 'vitest';
import {palettes,paletteById} from './appearance';
it('keeps a deterministic palette for legacy configurations',()=>{
 expect(paletteById('missing').id).toBe('black-green');
 expect(new Set(palettes.map(p=>p.id)).size).toBe(palettes.length);
 for(const p of palettes){expect(p.top).toMatch(/^#[0-9a-f]{6}$/);expect(p.bottom).toMatch(/^#[0-9a-f]{6}$/)}
});

it('uses the sampled reference pairs without substitute colors',()=>{expect(palettes.slice(1)).toEqual(samples);expect(palettes).toHaveLength(10)});
