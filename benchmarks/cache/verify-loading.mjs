import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..');
const compiler=process.env.TSRUST;
if(!compiler||process.version!=='v24.11.1')throw new Error('Set TSRUST and use Node v24.11.1.');
const run=(command,args)=>{const result=spawnSync(command,args,{cwd:root,encoding:'utf8',timeout:180000});if(result.error||result.status!==0)throw new Error(`${command}: ${result.error??result.stderr}`);return result.stdout.trim();};
const dir=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-cache-loading-'));
try{
 const source='examples/cache/serialized-loading.tsr';
 const counterpart='benchmarks/cache/serialized-loading.ts';
 const tsc=path.join(root,'benchmarks/runtime/node_modules/typescript/bin/tsc');
 run(process.execPath,[tsc,'--target','ES2022','--module','commonjs','--strict','--esModuleInterop','--skipLibCheck','--typeRoots',path.join(root,'benchmarks/runtime/node_modules/@types'),'--outDir',dir,counterpart]);
 const native=run(compiler,['run',source]);const node=run(process.execPath,[path.join(dir,'serialized-loading.js')]);
 if(native!=='serialized cache loading accepted'||node!==native)throw new Error('Loading contract mismatch.');
 const hash=file=>createHash('sha256').update(fs.readFileSync(path.resolve(root,file))).digest('hex');
 console.log(JSON.stringify({kind:'cache-serialized-loading',verifiedOnly:true,sourceCommit:run('git',['rev-parse','HEAD']),sourceSha256:hash(source),counterpartSha256:hash(counterpart),compilerSha256:hash(compiler),nodeVersion:process.version,harnessSha256:hash(fileURLToPath(import.meta.url)),sourceTreeDirty:run('git',['status','--porcelain']).length>0,expectedLoads:3,clients:2,requests:6,completionMessages:2,native,node,scope:'Successful load reuse, uncached failure retries, completion and rejection after close. Native additionally checks cancellation and closed reply handling. No timing or cancellation parity claim.'},null,2));
}finally{fs.rmSync(dir,{recursive:true,force:true});}
