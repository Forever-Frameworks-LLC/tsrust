import {test} from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {createRequire} from 'node:module';
import {execFileSync} from 'node:child_process';
import {assertABI} from './adapters.mjs';
import {expectedChecksum} from './oracle.mjs';
const require=createRequire(import.meta.url);
const ts=require('./node_modules/typescript');
const here=path.dirname(new URL(import.meta.url).pathname);
const root=path.resolve(here,'../..');
const ids=["higher_inline_reusable_int", "higher_borrowed_inline_factory_int", "higher_multiple_callbacks_int", "higher_mutate_array", "higher_borrowed_declared_factory_int", "higher_boolean_reusable", "recursive_callbacks", "higher_deep_array", "higher_number_reusable", "higher_recursive_tree", "callback_contracts", "higher_read_string", "higher_nested_consumer_int", "higher_distinct_origins_int", "higher_owned_record", "higher_declared_reusable_int", "owned_arguments"];
const manifest=JSON.parse(fs.readFileSync(path.join(here,'manifest.json'),'utf8'));

test('explicit borrowed callback workload migration preserves native ABI and scalar results',()=>{
 const compiler=path.join(root,'target/editor/debug/tsrust');
 const temporary=fs.mkdtempSync(path.join(root,'target/editor/callback-migration-'));
 try {
  for(const id of ids){
   const row=manifest.workloads.find(row=>row.id===id);
   assert.ok(row,id);
   const file=path.join(here,'workloads',id+'.tsr');
   const source=fs.readFileSync(file,'utf8');
   const emitted=execFileSync(compiler,['emit',file],{encoding:'utf8'});
   assertABI(emitted,row,{source});
   if(!['int','boolean'].includes(row.result)) continue;
   const reference=fs.readFileSync(path.join(here,'workloads',id+'.ts'),'utf8');
   const js=ts.transpileModule(reference,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.CommonJS}}).outputText;
   const evaluate=new Function('n','seed',js+'\nreturn workload(n,seed);');
   for(const [n,seed,flags] of [[0,7,[]],[2,7,[]],[17,0,['--release']]]){
    const expected=evaluate(n,seed);
    const checksum=row.result==='boolean'?Number(expected):expected;
    assert.equal(checksum,expectedChecksum(id,n,seed),id);
    const entry=path.join(temporary,'main.tsr');
    const rewritten=source.replace(/function main\s*\(\)\s*\{[\s\S]*$/,`function main(){console.log(workload(${n},${seed}));}`);
    assert.notEqual(rewritten,source,id);
    fs.writeFileSync(entry,rewritten);
    assert.equal(execFileSync(compiler,['run',entry,...flags],{encoding:'utf8'}),String(expected)+'\n',id);
   }
  }
 }finally{fs.rmSync(temporary,{recursive:true,force:true});}
});
