import {Monitor} from 'lucide-react';
import type {Game,VideoConfig} from './api';

export const defaultVideo:VideoConfig={enabled:false,resolution:'1920x1080',display:'borderless',targetFps:0,frameGeneration:'fsr3',upscaling:'off',resolutionScale:100,rendering:'standard'};
export function VideoSettings({game,locked,onChange,onOpenTest}:{game:Game;locked:boolean;onChange:(changes:Record<string,unknown>)=>void;onOpenTest?:()=>void}){
 const v=game.video??defaultVideo;
 const native=game.engine==='lbppc';
 const experimental=native&&v.rendering==='native-test';
 if(!native&&game.engine!=='rpcs3')return null;
 const update=(changes:Partial<VideoConfig>)=>onChange({video:changes});
 return <section id="video-settings" className="paper video-panel" aria-labelledby="video-heading">
  <div className="row"><Monitor size={24}/><h2 id="video-heading">Video settings</h2></div>
  <p>{native?'Native LBPPC settings.':'RPCS3 emulation settings.'} Changes apply next time you press Play.</p>
  <label className="check-field"><input type="checkbox" checked={v.enabled} disabled={locked} onChange={e=>update({enabled:e.target.checked})}/>Use these video settings when launching</label>
  <fieldset className="video-fields" disabled={!v.enabled||locked}>
   <legend className="sr-only">{native?'LBPPC':'RPCS3'} video options</legend>
   {native&&<label className="field">Rendering mode<select value={v.rendering??'standard'} onChange={e=>update({rendering:e.target.value as 'standard'|'native-test',targetFps:e.target.value==='native-test'?(v.targetFps===120?120:60):v.targetFps===120?60:v.targetFps})}><option value="standard">Standard</option><option value="native-test">Native rendering - experimental test</option></select></label>}
   <label className="field">{native?'Displayed frame rate':'Frame limit'}<select aria-describedby="fps-support" value={v.targetFps} onChange={e=>update({targetFps:Number(e.target.value)})}>
    <option value={0} disabled={experimental}>Keep engine setting</option><option value={30} disabled={experimental}>{native?'30 FPS - native':'30 FPS cap'}</option><option value={60}>{native?(experimental?'60 FPS - native rendering test':'60 FPS - frame generation'):'60 FPS cap'}</option><option value={120} disabled={native&&!experimental}>{native?(experimental?'120 FPS - native rendering test':'120 FPS - unavailable in Standard mode'):'120 FPS cap'}</option>
   </select></label>
   {native?<>
    <label className="field">Resolution<select value={v.resolution} onChange={e=>update({resolution:e.target.value})}>{['1280x720','1920x1080','2560x1440','3840x2160','2560x1080','3440x1440','3840x1600','3840x1080','4480x1260'].map(r=><option key={r} value={r}>{r.replace('x',' x ')}</option>)}</select></label>
    <label className="field">Display mode<select value={v.display} onChange={e=>update({display:e.target.value})}><option value="windowed">Windowed</option><option value="borderless">Borderless fullscreen</option><option value="fullscreen">Exclusive fullscreen</option></select></label>
    {v.targetFps===60&&!experimental&&<label className="field">Frame generation<select value={v.frameGeneration} onChange={e=>update({frameGeneration:e.target.value})}><option value="fsr3">FSR 3.1 - any supported GPU</option><option value="dlss">DLSS - GeForce RTX 40/50</option></select></label>}
    <label className="field">Upscaling<select value={v.upscaling} onChange={e=>update({upscaling:e.target.value})}><option value="off">Off</option><option value="fsr1">FSR 1</option></select></label>
   </>:<label className="field">Resolution scale<select value={v.resolutionScale} onChange={e=>update({resolutionScale:Number(e.target.value)})}>{[100,150,200,300].map(n=><option key={n} value={n}>{n}%</option>)}</select></label>}
  </fieldset>
  <p id="fps-support" className="hint">{experimental?'Experimental 60/120 native rendering requests: frame generation is OFF. Physics stays at 30 Hz; motion may still step at 30 Hz. Actual FPS, correct speed and stability are unverified. Requires the unmodified official LBPPC 0.1.5 Windows release and LBP3 US/Europe 1.28. Tests run offline with separate saves; your normal engine settings and saves stay separate.':native?'LBPPC runs the game at 30 steps per second. Its 60 FPS mode inserts generated frames. Native 60/120 FPS is not implemented in the current upstream release.':'A frame limit caps output; it does not unlock a 30 FPS game. Higher rates need compatible game patches and sufficient hardware. Resolution scaling uses the engine base resolution. Vblank timing is preserved.'}</p>
  {experimental&&<button className="button quiet" type="button" disabled={!onOpenTest} onClick={onOpenTest}>Open test logs and saves</button>}
  {!v.enabled&&<p className="hint">Your existing engine video preferences are used until you enable this option.</p>}
  {!native&&<label className="check-field"><input type="checkbox" disabled={locked} checked={game.ps3Compatibility??true} onChange={e=>onChange({ps3Compatibility:e.target.checked})}/>Use LBP starting profile (color-buffer effects)</label>}
  {!native&&<label className="check-field"><input type="checkbox" disabled={locked} checked={game.fullscreen} onChange={e=>onChange({fullscreen:e.target.checked})}/>Launch in fullscreen</label>}
 </section>;
}
