import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
export const cases=[
 ['ascii','([a-z]+)=([0-9]+)','a=12 b=3','${2}:$1 $$'],
 ['unicode','(é)(🦀)?','xé🦀 é','${2}/${1}'],
 ['absent-empty','(a)?()b','b ab','${1}:${2}'],
 ['no-match','z+','héllo','${0}'],
 ['empty-input','','','x'],
 ['zero-width','','é🦀','x'],
 ['adjacent-empty','a*','ab','x'],
 ['named-template','(?<word>[a-z]+)','cat dog','${word}:$word:$0:$$:${missing}'],
 ['split-boundaries',',',',a,,b,','|'],
 ['controls','(x)','x\n\t"\\','$1'],
];
const bytes=text=>Buffer.byteLength(text,'utf8');
const byteRange=(text,pair)=>pair===undefined?null:[bytes(text.slice(0,pair[0])),bytes(text.slice(0,pair[1]))];
function matches(pattern,text){
 const rows=[];
 for(const match of text.matchAll(new RegExp(pattern,'dgu'))){
  // Rust's nonoverlapping iterator suppresses an empty match immediately after
  // another match ending at that position; JS matchAll reports that extra match.
  if(match[0]===''&&rows.length&&rows.at(-1).index+rows.at(-1)[0].length===match.index)continue;
  rows.push(match);
 }
 return rows;
}
function expand(template,match){
 return template.replace(/\$\$|\$\{([^}]+)\}|\$([A-Za-z0-9_]+)/g,(token,braced,bare)=>{
  if(token==='$$')return '$';
  const key=braced??bare;
  return /^\d+$/.test(key)?match[Number(key)]??'':match.groups?.[key]??'';
 });
}
function replacement(text,rows,template,all){let out='',start=0;for(const match of all?rows:rows.slice(0,1)){out+=text.slice(start,match.index)+expand(template,match);start=match.index+match[0].length;}return out+text.slice(start);}
export function observation([id,pattern,text,template]){
 const rows=matches(pattern,text),ranges=rows.map(match=>byteRange(text,match.indices[0]));
 const capturesAll=rows.map(match=>Array.from(match.indices,pair=>byteRange(text,pair)));
 let start=0;const split=[];for(const range of ranges){split.push([start,range[0]]);start=range[1];}split.push([start,bytes(text)]);
 return {kind:'correctness',case:id,isMatch:rows.length>0,find:ranges[0]??null,findAll:ranges,captures:capturesAll[0]??null,capturesAll,replace:replacement(text,rows,template,false),replaceAll:replacement(text,rows,template,true),split,search:ranges[0]?.[0]??-1};
}
export function validateCorrectness(rows){
 assert.equal(rows.length,cases.length+1);
 for(let index=0;index<cases.length;index++)assert.deepEqual(rows[index],observation(cases[index]));
 assert.deepEqual(rows.at(-1),{kind:'contracts',unsupportedPatterns:3,explicitLimitRejected:true,escapeRoundTrip:true,ownedRangesSurviveOwners:true});
}
const rangeWeight=range=>range===null?0:range[1]-range[0]+1;
export function profileExpected(kernel,scenario,size){
 const pattern=scenario==='scaling'?'a+z|a':scenario==='unicode'?'(é)(🦀)?':'([a-z]+)=([0-9]+)';
 const text=scenario==='scaling'?'a'.repeat(size):scenario==='unicode'?'é🦀 é '.repeat(size):scenario==='no-match'?'!'.repeat(size):'key=123 other=4 '.repeat(size);
 const value=observation(['profile',pattern,text,'${2}:${1}']);
 if(kernel.startsWith('compile_'))return 3;
 if(kernel.startsWith('is_match_'))return Number(value.isMatch);
 if(kernel.startsWith('find_all_')||kernel.startsWith('scale_find_all_'))return value.findAll.reduce((sum,range)=>sum+rangeWeight(range),0);
 if(kernel.startsWith('find_'))return rangeWeight(value.find);
 if(kernel.startsWith('captures_all_'))return value.capturesAll.flat().reduce((sum,range)=>sum+rangeWeight(range),0);
 if(kernel.startsWith('captures_'))return value.captures?.reduce((sum,range)=>sum+rangeWeight(range),0)??0;
 if(kernel.startsWith('replace_all_'))return bytes(value.replaceAll);
 if(kernel.startsWith('replace_'))return bytes(value.replace);
 if(kernel.startsWith('split_'))return value.split.reduce((sum,range)=>sum+rangeWeight(range),0);
 throw new Error(`Unknown kernel ${kernel}`);
}
export function validateProfile(rows,instrumented){
 const names=['compile_direct','compile_adapter','is_match_direct','is_match_adapter','find_direct','find_adapter','find_all_direct','find_all_ranges','find_all_strings','captures_direct','captures_locations','captures_adapter','captures_all_direct','captures_all_adapter','replace_direct','replace_adapter','replace_all_direct','replace_all_adapter','split_direct','split_ranges'];
 const expected=new Set(names.flatMap(kernel=>['ascii','unicode','no-match'].map(scenario=>`${kernel}:${scenario}:32`)).concat([64,256,1024].flatMap(size=>['direct','ranges'].map(kind=>`scale_find_all_${kind}:scaling:${size}`))));
 for(const row of rows){const id=`${row.kernel}:${row.scenario}:${row.size}`;assert(expected.delete(id),id);const checksum=profileExpected(row.kernel,row.scenario,row.size);if(instrumented){assert.equal(row.kind,'allocations');assert.equal(row.checksum,checksum);for(const field of ['allocations','reallocations','requested','peak'])assert(Number.isSafeInteger(row[field])&&row[field]>=0);}else{assert.equal(row.kind,'timing');assert.equal(row.durations_ns.length,31);assert.equal(row.checksums.length,31);assert(Number.isSafeInteger(row.rounds)&&row.rounds>0);assert(row.durations_ns.every(value=>Number.isSafeInteger(value)&&value>=0));assert(row.checksums.every(value=>value===checksum*row.rounds));}}
 assert.equal(expected.size,0);
}
export function selfCheck(){
 assert.deepEqual(observation(cases[1]).findAll,[[1,7],[8,10]]);
 assert.deepEqual(observation(cases[2]).capturesAll,[[[0,1],null,[0,0]],[[2,4],[2,3],[3,3]]]);
 assert.deepEqual(observation(cases[5]).findAll,[[0,0],[2,2],[6,6]]);
 assert.deepEqual(observation(cases[6]).findAll,[[0,1],[2,2]]);
 assert.deepEqual(observation(cases[8]).split,[[0,0],[1,2],[3,3],[4,5],[6,6]]);
 assert.equal(observation(cases[7]).replaceAll,'cat:cat:cat:$: dog:dog:dog:$:');
 assert.equal(observation(cases[3]).replaceAll,'héllo');
 assert.equal(profileExpected('find_all_ranges','ascii',32),512);
 assert.equal(profileExpected('scale_find_all_ranges','scaling',64),128);
 const broken=structuredClone(cases.map(observation));broken[0].find[0]++;assert.throws(()=>validateCorrectness([...broken,{kind:'contracts'}]));
}
if(process.argv[1]===fileURLToPath(import.meta.url)){selfCheck();console.log('Regex UTF-8 range/template oracle controls passed');}
