import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertLabeledControlSource} from './labeled-control-source-contract.mjs';
import {labeledControlIds,expectedLabeledControlValues} from './labeled-control-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='labeled-control');
test('labeled control registration preserves prior rows and guards complete scenarios',()=>{
 
 assert.deepEqual(rows.map(row=>row.id),labeledControlIds);
 for(const row of rows){assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'other');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertLabeledControlSource(source,row,kind);
   for(const text of ["while (i < n)", "outer:for(let row=0;row<4;row++,updates++)", "if(row===skip)continue outer;", "if(row===3)break outer;", "trace+=String(row)+String(col);", "again:while(count<base%4+1)", "if(x===1)continue again;", "postLoop:do{post++;for(;;){continue postLoop;}}while(post<base%3+1);", "done:{if(base%2===0)break done;selected=2;}", "first:second:for(const value of [base,base+1])", "break first;", "output.push([trace,String(updates),String(count),String(post),String(selected),String(once),input+String(base)]);", "return output;"]){
    const changed=replaceCodeSnippet(source, text,'');const key=kind==='native'?'sourceSha256':'counterpartSha256';
    assert.throws(()=>assertLabeledControlSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
   }
   assert.throws(()=>assertLabeledControlSource(source,{...row,operationsPerIteration:8},kind));assert.throws(()=>assertLabeledControlSource(source+' ',row,kind));
  }
 }
 for(const file of ['./run.mjs','./profile-native.mjs']){const source=read(file);assert.match(source,/assertLabeledControlSource\(source/);assert.ok(source.includes("'labeled-control-oracle.mjs'"));}
});
test('independent labeled control oracle retains cells and agrees with platform',()=>{
 const expected=[['101112202122','3','1','1','1','0','ab0'],['000102202122','3','2','2','2','13','ab13']];
 assert.deepEqual(expectedLabeledControlValues('labeled_control',2,0,'ab'),expected);
 const observation=observeSliceTree(expected);assert.deepEqual(expectedExactResult('labeled_control',2,0,'ab'),observation);assert.equal(expectedChecksum('labeled_control',2,0,'ab'),observation.checksum);
 assert.deepEqual(expectedLabeledControlValues('labeled_control',0,0,''),[]);
 for(const input of ['dynamic é🙂 text','abc','']){const expected=expectedLabeledControlValues('labeled_control',2048,7,input);
  for(let i=0;i<2048;i++){const base=(i*13+7)%97,skip=base%3;let trace='',updates=0;
   outer:for(let row=0;row<4;row++,updates++){for(let col=0;col<3;col++){if(row===skip)continue outer;if(row===3)break outer;trace+=row.toString()+col;}}
   let count=0;again:while(count<base%4+1){count++;for(const x of [1,2]){if(x===1)continue again;count+=10;}}
   let post=0;postLoop:do{post++;for(;;){continue postLoop;}}while(post<base%3+1);
   let selected=1;done:{if(base%2===0)break done;selected=2;}
   let once=0;first:second:for(const value of [base,base+1]){once+=value;for(;;){break first;}}
   assert.deepEqual(expected[i],[trace,String(updates),String(count),String(post),String(selected),String(once),input+base]);
  }
 }
});
