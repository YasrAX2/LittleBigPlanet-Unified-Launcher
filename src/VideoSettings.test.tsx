import {it,expect} from 'vitest';
import {renderToStaticMarkup} from 'react-dom/server';
import {VideoSettings,defaultVideo} from './VideoSettings';
import type {Game} from './api';
const game=(engine:string):Game=>({id:'lbp3',engine,data:'',update:'',titleId:'',region:'',version:'',favorite:false,seconds:0,lastPlayed:0,fullscreen:false,artwork:'',video:{...defaultVideo,enabled:true}});
it('does not advertise an unsupported native 120 FPS mode',()=>{
 const html=renderToStaticMarkup(<VideoSettings game={game('lbppc')} locked={false} onChange={()=>{}}/>);
 expect(html).toContain('value="120" disabled=""');expect(html).toContain('60 FPS - frame generation');expect(html).toContain('30 steps per second');
});
it('offers genuine RPCS3 caps and explains the patch requirement',()=>{
 const html=renderToStaticMarkup(<VideoSettings game={game('rpcs3')} locked={false} onChange={()=>{}}/>);
 expect(html).toContain('120 FPS cap');expect(html).not.toContain('value="120" disabled');expect(html).toContain('does not unlock a 30 FPS game');
});

it('offers 60/120 native tests separately from standard generated frames',()=>{
 const g=game('lbppc');g.video={...g.video!,rendering:'native-test',targetFps:120};
 const html=renderToStaticMarkup(<VideoSettings game={g} locked={false} onChange={()=>{}}/>);
 expect(html).toContain('120 FPS - native rendering test');expect(html).not.toContain('value="120" disabled');
 expect(html).toContain('frame generation is OFF');expect(html).toContain('Physics stays at 30 Hz');expect(html).toContain('separate saves');
 expect(html).not.toContain('DLSS - GeForce');
});
