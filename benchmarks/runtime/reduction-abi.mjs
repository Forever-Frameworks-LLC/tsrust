// Benchmark-only fail-closed type binding. Called on actual CLI output before rustc.
import crypto from 'node:crypto';
import {reductionFields,isReduction} from './reduction-observation.mjs';
const hash=text=>crypto.createHash('sha256').update(text).digest('hex');
const compact=text=>text.replace(/\s+/g,'');
const fail=id=>{throw Error('Reduction emitted ABI/schema changed: '+id);};
export function assertReductionSource(source,workload){
 if(!isReduction(workload)||typeof source!=='string')fail(workload.id);
 const signatures=[...source.matchAll(/function\s+workload\s*\(([^)]*)\)\s*:\s*([A-Za-z0-9_\[\]\s]+)\s*\{/g)];
 if(signatures.length!==1||compact(signatures[0][1])!=='n:int,seed:int'||compact(signatures[0][2])!==workload.result)fail(workload.id);
 if(workload.result.includes('Row')){
  if(!workload.recordSchema||workload.recordSchema.record!=='Row'||JSON.stringify(workload.recordSchema.fields)!==JSON.stringify(reductionFields.map(([name,type])=>({name,type}))))fail(workload.id);
  const declarations=[...source.matchAll(/type\s+([A-Za-z0-9_]+)\s*=\s*\{([^}]+)\}\s*;/g)];
  if(declarations.length!==1||declarations[0][1]!=='Row')fail(workload.id);
  const fields=declarations[0][2].split(';').map(s=>s.trim()).filter(Boolean).map(s=>{const m=/^([A-Za-z0-9_]+)\s*:\s*([A-Za-z0-9_\[\]\s]+)$/.exec(s);if(!m)fail(workload.id);return [m[1],compact(m[2])];});
  if(JSON.stringify(fields)!==JSON.stringify(reductionFields))fail(workload.id);
 }
}
export function verifyReductionABI(emitted,workload,source){
 assertReductionSource(source,workload);if(typeof emitted!=='string'||!emitted.includes('fn main()'))fail(workload.id);
 const functions=[...emitted.matchAll(/fn\s+function0\s*\(([^)]*)\)\s*->\s*([^\n{]+)\s*\{/g)];
 if(functions.length!==1||compact(functions[0][1])!=='value0:i64,value1:i64')fail(workload.id);
 const result=compact(functions[0][2]),aliases=new Map(),containers=new Set();
 for(const m of emitted.matchAll(/type\s+(RecordArray\d+)\s*=\s*([^;]+);/g)){if(aliases.has(m[1]))fail(workload.id);aliases.set(m[1],compact(m[2]));}
 const records=new Map();
 for(const m of emitted.matchAll(/struct\s+(Record\d+)\s*\{([^{}]*)\}/g)){
  if(records.has(m[1]))fail(workload.id);
  // Split fields at depth zero so generic type commas cannot alter field count.
  const parts=[];let level=0,start=0;for(let i=0;i<m[2].length;i++){if(m[2][i]==='<')level++;if(m[2][i]==='>')level--;if(level<0)fail(workload.id);if(m[2][i]===','&&level===0){parts.push(m[2].slice(start,i));start=i+1;}}parts.push(m[2].slice(start));
  const fields=parts.map(s=>s.trim()).filter(Boolean).map(s=>{const n=/^(field\d+)\s*:\s*(.+)$/.exec(s);if(!n)fail(workload.id);return {name:n[1],type:compact(n[2])};});records.set(m[1],fields);
 }
 let record;
 function check(rust,type,seen=new Set()){
  if(seen.has(rust))fail(workload.id);
  if(type.endsWith('[]')){
   let expanded=rust;if(aliases.has(rust)){seen=new Set(seen).add(rust);expanded=aliases.get(rust);}
   const m=/^(Vec|Array|NumericArray)<(.+)>$/.exec(expanded);if(!m)fail(workload.id);containers.add(m[1]);
   if(m[1]==='Array'&&!/\b(?:enum|struct)\s+Array\s*</.test(emitted))fail(workload.id);
   if(m[1]==='NumericArray'&&!/\b(?:enum|struct)\s+NumericArray\s*</.test(emitted))fail(workload.id);
   check(m[2],type.slice(0,-2),seen);return;
  }
  if(type==='Row'){
   const fields=records.get(rust);if(!fields||fields.length!==reductionFields.length||record&&record.name!==rust)fail(workload.id);
   if(fields.some((f,i)=>f.name!=='field'+i))fail(workload.id);
   record={name:rust,fields:fields.map((f,i)=>({source:reductionFields[i][0],rust:f.name,type:f.type,declaredType:reductionFields[i][1]}))};
   for(let i=0;i<fields.length;i++)check(fields[i].type,reductionFields[i][1],new Set(seen).add(rust));return;
  }
  if(rust!==({int:'i64',number:'f64',boolean:'bool',string:'String'})[type])fail(workload.id);
  if(type==='string'&&!/type\s+String\s*=\s*std::borrow::Cow\s*<\s*'static\s*,\s*str\s*>\s*;/.test(emitted))fail(workload.id);
 }
 check(result,workload.result);
 return {protocol:'actual-emission-reduction-abi-v1',resultType:result,record:record??null,containers:[...containers].sort(),emittedSha256:hash(emitted),sourceSha256:hash(source)};
}
export function pruneObserverContainers(observer,binding,trait){
 for(const [container,header]of [['Array',`impl<T:${trait}> ${trait} for Array<T> {`],['NumericArray',`impl<T:${trait}+Copy+Default> ${trait} for NumericArray<T> {`]]){
  if(binding.containers.includes(container))continue;
  const start=observer.indexOf(header);if(start<0||observer.indexOf(header,start+1)>=0)throw Error('Reduction observer container hook changed');
  // Both reviewed observers use one complete implementation ending at newline }.
  const open=observer.indexOf('{',start);let level=1,end=open+1;while(end<observer.length&&level){if(observer[end]==='{')level++;else if(observer[end]==='}')level--;end++;}if(level)throw Error('Reduction observer container hook changed');observer=observer.slice(0,start)+observer.slice(end);
 }
 return observer;
}
export function recordObserver(binding){
 if(!binding.record)return '';
 return `impl ReductionTree for ${binding.record.name} { fn visit(&self,v:&mut dyn FnMut(ReductionToken)){v(ReductionToken::RecordStart);${binding.record.fields.map(f=>`self.${f.rust}.visit(v);`).join('')}v(ReductionToken::RecordEnd);} }`;
}
