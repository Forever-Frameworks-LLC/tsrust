import {test} from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {createRequire} from 'node:module';
import {assertABI} from './adapters.mjs';
import {assertPathRelativeSource,pathRelativeIds,pathRelativeChecksum,pathRelativeValues} from './path-relative-source-contract.mjs';
import {expectedChecksum} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const require=createRequire(import.meta.url);
const ts=require('./node_modules/typescript');
const here=path.dirname(new URL(import.meta.url).pathname);
const root=path.resolve(here,'../..');
const compiler=process.env.TSRUST || path.join(root,'target/editor/debug/tsrust');
const manifest=JSON.parse(fs.readFileSync(path.join(here,'manifest.json')));
for(const id of pathRelativeIds) test(id+' preserves source, ABI and independent results',()=>{
    const w=manifest.workloads.find(row=>row.id===id);
    assert.equal(reportCategory(id),'paths');
    assert.equal(manifest.workloads.at(-1).id,id);assert.equal(manifest.workloads.at(-2).id,'generic_callable_replacement');
    const source=fs.readFileSync(path.join(here,w.source),'utf8');
    const counterpart=fs.readFileSync(path.join(here,w.counterpart),'utf8');
    for(const [text,hash] of [[source,w.sourceSha256],[counterpart,w.counterpartSha256]]){
        assert.equal(createHash('sha256').update(text).digest('hex'),hash);
        assertPathRelativeSource(text,w);
        assert.throws(()=>assertPathRelativeSource(text.replace('i = i + 1','i = i + 2'),w));
        assert.throws(()=>assertPathRelativeSource(text.replace('std:path','std:fs').replace('node:path','node:fs'),w));
    }
    assertABI(execFileSync(compiler,['emit',path.join(here,w.source)],{encoding:'utf8'}),w,{source});
    const program=ts.createProgram([path.join(here,w.counterpart)],{strict:true,noEmit:true,target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.CommonJS,types:[],skipLibCheck:true});
    assert.equal(ts.getPreEmitDiagnostics(program).length,0,'matched counterpart must typecheck');
    const js=ts.transpileModule(counterpart,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.CommonJS}}).outputText;
    const evaluate=new Function('require','n','seed',js+';return workload(n,seed);');
    const dir=fs.mkdtempSync(path.join(root,'target/editor/path-relative-'));
    try{for(const [n,seed] of [[0,7],[2,7],[17,0]]){

        const expected=pathRelativeValues(id,n,seed);
        assert.deepEqual(evaluate(require,n,seed),expected);
        assert.equal(expectedChecksum(id,n,seed),pathRelativeChecksum(id,n,seed));
        const file=path.join(dir,'main.tsr');
        fs.writeFileSync(file,source.slice(0,source.indexOf('function main()'))+`function main(){const values=workload(${n},${seed});for(const value of values){console.log(value);}}`);
        assert.equal(execFileSync(compiler,['run',file],{encoding:'utf8'}),expected.map(s=>s+'\n').join(''));
    }}finally{fs.rmSync(dir,{recursive:true,force:true});}
});
