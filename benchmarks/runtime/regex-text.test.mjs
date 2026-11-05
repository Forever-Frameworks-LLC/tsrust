import {assertABI} from './adapters.mjs';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {regexTextSpecs,regexTextChecksum,assertRegexTextSource} from './regex-text-contract.mjs';
import {selfCheck as admissionOracleControls} from './admission/regex-text/oracle.mjs';

export function selfCheck() {
  admissionOracleControls();
  assert.equal(Object.keys(regexTextSpecs).length,13);
  const observed={
    regex_compile:1,regex_escape:14,regex_is_match:1,regex_find:5,regex_find_all:19,
    regex_captures:14,regex_captures_all:56,regex_replace:10,regex_replace_all:12,
    regex_split:28,string_regex_match:14,string_regex_match_all:56,string_regex_search:1,
  };
  const manifest=JSON.parse(fs.readFileSync(new URL('./manifest.json',import.meta.url))).workloads.filter(row=>row.family==='regex-text');assert.equal(manifest.length,13);
  for(const[id,spec]of Object.entries(regexTextSpecs)) {
    const source=fs.readFileSync(new URL(`./workloads/${id}.tsr`,import.meta.url),'utf8');
    const counterpart=fs.readFileSync(new URL(`./workloads/${id}.ts`,import.meta.url),'utf8');
    assert(fs.statSync(new URL(`./parity/${id}.rs`,import.meta.url)).isFile());
    const row=manifest.find(row=>row.id===id);assert(row,id);
    assert.equal(assertABI(fs.readFileSync(new URL(row.rustReference,import.meta.url),'utf8'),row).fallible,false,id);
    assertRegexTextSource(source,row,'native');assertRegexTextSource(counterpart,row,'typescript');
    for(const[n,seed]of[[0,0],[1,0],[3,7],[2000,19]])assert.equal(regexTextChecksum(id,n,seed,spec.input),seed+n*observed[id]);
    assert.throws(()=>regexTextChecksum(id,1,0,'wrong input'));
    const stepChanged=source.replace('i++;','i += 2;');
    assert.throws(()=>assertRegexTextSource(stepChanged,{...row,sourceSha256:createHash('sha256').update(stepChanged).digest('hex')},'native'));
    const operationChanged=source.replace(spec.native,'throw "missing operation";');
    assert.throws(()=>assertRegexTextSource(operationChanged,{...row,sourceSha256:createHash('sha256').update(operationChanged).digest('hex')},'native'));
    assert.equal(createHash('sha256').update(source).digest('hex').length,64);
  }
  const text='a=12 b=3';
  const matches=Array.from(text.matchAll(/([a-z]+)=([0-9]+)/dg));
  const sum=range=>range[0]+range[1]+1;
  assert.equal(matches.reduce((total,value)=>total+sum(value.indices[0]),0),observed.regex_find_all);
  assert.equal(matches.flatMap(value=>value.indices).reduce((total,value)=>total+sum(value),0),observed.regex_captures_all);
  assert.equal(Buffer.byteLength('a\\+b\\[é\\]\\.\\*','utf8'),observed.regex_escape);
  assert.equal(text.replace(/([a-z]+)=([0-9]+)/d,'$2:$1 $$').length,observed.regex_replace);
  assert.equal(text.replace(/([a-z]+)=([0-9]+)/dg,'$2:$1 $$').length,observed.regex_replace_all);
}
if(process.argv[1]===fileURLToPath(import.meta.url)){selfCheck();console.log('Regex/text workload source and independent oracle controls passed');}
