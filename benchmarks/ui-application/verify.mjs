// Untimed source/theme parity. A later timing run needs no new oracle implementation.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
const here=path.dirname(fileURLToPath(import.meta.url));
assert(process.env.TSRUST,'Set TSRUST to the current compiler');
const root=path.resolve(here,'../../examples/ui-application');
const scratch=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-ui-application-'));
function run(command,args){const result=spawnSync(command,args,{encoding:'utf8',timeout:180000,env:{...process.env,CARGO_INCREMENTAL:'0',CARGO_BUILD_JOBS:'1'}});assert.equal(result.status,0,result.stderr||result.stdout||String(result.error));return result.stdout;}
try {
 run(process.execPath,[path.join(here,'../runtime/node_modules/typescript/bin/tsc'),'--target','es2022','--lib','es2022','--skipLibCheck','--outDir',scratch,path.join(here,'reference.ts')]);
 const expected=JSON.parse(run(process.execPath,[path.join(scratch,'reference.js'),path.join(root,'tsrust.config.json')]));
 const binary=path.join(scratch,'inspect');run(process.env.TSRUST,['build',path.join(root,'inspect.tsr'),'-o',binary]);const html=run(binary,[]);
 for(const [key,value] of Object.entries(expected)) {
  const property={background:'--ui-background',border:'--ui-border',card:'--ui-card',foreground:'--ui-foreground',radius:'border-radius',base:'--ui-text-base',largePaddingX:'padding-left',largePaddingY:'padding-top'}[key];
  assert(html.includes(`${property}:${value}${typeof value==='number'?'px':''}`),`${key}: ${value}`);
 }
 assert(html.includes('Count: 0'));assert(html.includes('Increment'));
 console.log(JSON.stringify({status:'pass',engines:['native','node-tsc'],timed:false,observations:expected}));
 for(const file of ['reference.ts','../../examples/ui-application/inspect.tsr','../../examples/ui-application/src/app/page.tsrx','../../examples/ui-application/src/components/ui/button.tsrx','../../examples/ui-application/src/components/ui/card.tsrx','../../examples/ui-application/tsrust.config.json']) {
  console.log(JSON.stringify({source:file,sha256:createHash('sha256').update(fs.readFileSync(path.resolve(here,file))).digest('hex')}));
 }
} finally {fs.rmSync(scratch,{recursive:true,force:true});}
