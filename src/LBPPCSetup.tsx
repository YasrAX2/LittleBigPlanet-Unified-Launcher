import {useEffect,useState} from 'react';
import {call} from './api';

type Account={signedIn:boolean;username:string;online:boolean};
type Response=Account&{complete?:boolean;message?:string};
export function LBPPCSetup({installed,locked,onBuild,onChanged}:{installed:boolean;locked:boolean;onBuild:()=>void;onChanged:()=>Promise<void>}){
 const [account,setAccount]=useState<Account>();
 const [working,setWorking]=useState('');
 const [message,setMessage]=useState('');
 const [error,setError]=useState(false);
 useEffect(()=>{let stopped=false;if(installed)call<Account>('lbppc_setup',{action:'status'}).then(value=>{if(!stopped)setAccount(value)}).catch(()=>{});return()=>{stopped=true}},[installed]);
 async function action(action:string){
  setWorking(action);setError(false);setMessage(action==='signin'?'Finish signing in or creating your account in the official browser page.':'Checking LBPPC…');
  try{
   const result=await call<Response>('lbppc_setup',{action});
   if(action==='check'){setError(!result.complete);setMessage(result.message??'File check finished.');}
   else{setAccount(result);setMessage(result.signedIn?'Account saved. LBPPC will sign in automatically when you play.':'Signed out.');}
   await onChanged();
  }catch(e){setError(true);setMessage(String(e));}finally{setWorking('');}
 }
 return <section className="lbppc-setup" aria-label="LBPPC setup">
  <h3>Build and set up LBPPC</h3>
  <p>Use the ready-made engine above, or compile the official source on this PC.</p>
  <div className="actions"><button className="button" disabled={locked||!!working} onClick={onBuild}>Build LBPPC from source</button><button className="button quiet" onClick={()=>void call('open_location',{kind:'buildLogs',id:null}).catch(e=>{setError(true);setMessage(String(e))})}>Open build logs</button></div>
  {installed&&<>
   <h3>Game setup and multiplayer</h3>
   <p>Select LBP3’s PS4 game/update folders, then check and apply setup. A build cannot replace missing game files.</p>
   <div className="actions"><button className="button" disabled={locked||!!working} onClick={()=>void action('check')}>Check and set up game</button></div>
   <p>{account?.signedIn?`Saved account: ${account.username||'LBPPC account'}.`:'Sign in once through lbppc.com, or create your account on its sign-in page.'}</p>
   <div className="actions"><button className="button" disabled={locked||!!working} onClick={()=>void action('signin')}>{account?.signedIn?'Sign in with another account':'Sign in / Create account'}</button>{account?.signedIn&&<button className="button quiet" disabled={locked||!!working} onClick={()=>void action('signout')}>Sign out</button>}{working==='signin'&&<button className="button quiet" onClick={()=>void call('cancel_lbppc_signin').catch(e=>{setError(true);setMessage(String(e))})}>Cancel sign-in</button>}</div>
   <p className="hint">Uses the official LBPPC community server. Passwords stay in your browser. Windows protects the remembered sign-in; it is not stored in Patchwork Pod backups. Friends and co-op invitations are managed in LBPPC.</p>
  </>}
  {message&&<p role={error?'alert':'status'} className={error?'inline-error':'hint'}>{message}</p>}
 </section>;
}
