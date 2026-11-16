import {describe,it,expect} from 'vitest';
import {catalog,bytes,playtime,progress,activeTask,type Task} from './api';
describe('library and real progress',()=>{
 it('keeps all three games, including unavailable native ports',()=>expect(catalog.map(g=>g.id)).toEqual(['lbp1','lbp2','lbp3']));
 it('uses real bytes and caps invalid progress',()=>{const t={downloaded:25,total:100,status:'Downloading'} as Task;expect(progress(t)).toBe(25);expect(progress({...t,total:0})).toBe(0);expect(progress({...t,downloaded:101})).toBe(100);expect(activeTask(t)).toBe(true);expect(activeTask({...t,status:'Failed'})).toBe(false)});
 it('formats local playtime and sizes',()=>{expect(playtime(0)).toBe('Not played yet');expect(playtime(3660)).toBe('1 h 1 min');expect(bytes(1048576)).toBe('1.0 MB')});
});
