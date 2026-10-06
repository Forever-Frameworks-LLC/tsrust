// No native ABI is presumed: this guard consumes actual emitted compiler bytes.
import crypto from 'node:crypto';
import {assertConcatPolicy,concatFields} from './concat-observation.mjs';
import {pruneObserverContainers} from './reduction-abi.mjs';
const hash=text=>crypto.createHash('sha256').update(text).digest('hex'),compact=text=>text.replace(/\s+/g,'');
const fail=id=>{throw Error('Concat emitted ABI/schema changed: '+id);};
export function assertConcatSource(source,w){
 assertConcatPolicy(w);if(typeof source!=='string'||!/^([a-f0-9]{64})$/.test(w.concatSourceSha256??'')||hash(source)!==w.concatSourceSha256)fail(w.id);
 const declarations=[...source.matchAll(/function\s+workload\s*\(([^)]*)\)\s*:\s*([A-Za-z0-9_\[\]\s]+)\s*\{/g)],parameters='n:int,seed:int'+(w.inputText===undefined?'':',input:string');
 if(declarations.length!==1||compact(declarations[0][1])!==parameters||compact(declarations[0][2])!==w.result)fail(w.id);
 if(w.recordSchema){const declarations=[...source.matchAll(/type\s+([A-Za-z0-9_]+)\s*=\s*\{([^}]+)\}\s*;/g)];if(declarations.length!==1||declarations[0][1]!==w.recordSchema.record)fail(w.id);const fields=declarations[0][2].split(';').map(s=>s.trim()).filter(Boolean).map(s=>{const m=/^([A-Za-z0-9_]+)\s*:\s*([A-Za-z0-9_\[\]\s]+)$/.exec(s);if(!m)fail(w.id);return [m[1],compact(m[2])];});if(JSON.stringify(fields)!==JSON.stringify(concatFields[w.recordSchema.record]))fail(w.id);}
}
export function assertConcatTSSource(source,w){assertConcatPolicy(w);if(typeof source!=='string'||!/^([a-f0-9]{64})$/.test(w.concatTSSha256??'')||hash(source)!==w.concatTSSha256)fail(w.id);if(w.concatTSAlphaMap&&(w.concatTSAlphaMap.from!=='ConcatBenchmarkNode'||w.concatTSAlphaMap.to!=='Node'||w.concatDomain!=='recursive_record'))fail(w.id);}
export function verifyConcatABI(emitted,w,source){
 assertConcatSource(source,w);if(typeof emitted!=='string'||!emitted.includes('fn main()'))fail(w.id);
 const declarations=[...emitted.matchAll(/fn\s+function0\s*\(([^)]*)\)\s*->\s*([^\n{]+)\s*\{/g)],parameters='value0:i64,value1:i64'+(w.inputText===undefined?'':',value2:&String');
 if(declarations.length!==1||compact(declarations[0][1])!==parameters)fail(w.id);
 const result=compact(declarations[0][2]),aliases=new Map(),customArrays=new Map(),records=new Map(),bindings=new Map(),containers=new Set(),checked=new Set();
 for(const m of emitted.matchAll(/type\s+(RecordArray\d+)\s*=\s*([^;]+);/g)){if(aliases.has(m[1]))fail(w.id);aliases.set(m[1],compact(m[2]));}
 for(const m of emitted.matchAll(/struct\s+(RecordArray\d+)\s*\(\s*(Vec\s*<\s*Record\d+\s*>)\s*\)\s*;/g)){if(customArrays.has(m[1])||aliases.has(m[1]))fail(w.id);customArrays.set(m[1],compact(m[2]));}
 for(const m of emitted.matchAll(/struct\s+(Record\d+)\s*\{([^{}]*)\}/g)){
  if(records.has(m[1]))fail(w.id);const parts=[];let level=0,start=0;for(let i=0;i<m[2].length;i++){if(m[2][i]==='<')level++;if(m[2][i]==='>')level--;if(level<0)fail(w.id);if(m[2][i]===','&&level===0){parts.push(m[2].substring(start,i));start=i+1;}}if(level!==0)fail(w.id);parts.push(m[2].substring(start));
  records.set(m[1],parts.map(s=>s.trim()).filter(Boolean).map(s=>{const n=/^(field\d+)\s*:\s*(.+)$/.exec(s);if(!n)fail(w.id);return {name:n[1],type:compact(n[2])};}));
 }
 const emittedCompact=compact(emitted);
 function check(rust,type,aliasSeen=new Set()){
  if(type.endsWith('[]')){
   let expanded=rust;if(aliases.has(rust)){if(aliasSeen.has(rust))fail(w.id);return check(aliases.get(rust),type,new Set(aliasSeen).add(rust));}
   if(customArrays.has(rust)){expanded=customArrays.get(rust);const exact=`implstd::ops::Dereffor${rust}{typeTarget=${expanded};fnderef(&self)->&Self::Target{&self.0}}`;if(!emittedCompact.includes(exact))fail(w.id);containers.add(rust);}
   const m=/^(Vec|Array|NumericArray)<(.+)>$/.exec(expanded);if(!m)fail(w.id);containers.add(m[1]);
   if(m[1]!=='Vec'&&!new RegExp('\\b(?:enum|struct)\\s+'+m[1]+'\\s*<').test(emitted))fail(w.id);
   check(m[2],type.substring(0,type.length-2),aliasSeen);return;
  }
  if(Object.hasOwn(concatFields,type)){
   if(bindings.has(type)&&bindings.get(type).name!==rust)fail(w.id);const fields=records.get(rust),schema=concatFields[type];if(!fields||fields.length!==schema.length||fields.some((f,i)=>f.name!=='field'+i))fail(w.id);
   if(!bindings.has(type))bindings.set(type,{source:type,name:rust,fields:fields.map((f,i)=>({source:schema[i][0],rust:f.name,type:f.type,declaredType:schema[i][1]}))});
   const key=rust+'|'+type;if(checked.has(key))return;checked.add(key);for(let i=0;i<schema.length;i++)check(fields[i].type,schema[i][1],new Set());return;
  }
  if(rust!==({int:'i64',number:'f64',boolean:'bool',string:'String'})[type])fail(w.id);
  if(type==='string'&&!/type\s+String\s*=\s*std::borrow::Cow\s*<\s*'static\s*,\s*str\s*>\s*;/.test(emitted))fail(w.id);
 }
 check(result,w.result);
 return {protocol:'actual-emission-concat-abi-v1',resultType:result,sourceResult:w.result,maxDepth:16,records:[...bindings.values()],containers:[...containers].sort(),customArrays:[...customArrays].filter(([name])=>containers.has(name)).map(([name,type])=>({name,type})),emittedSha256:hash(emitted),sourceSha256:hash(source)};
}
export function concatRustObserver(observer,binding){
 observer=pruneObserverContainers(observer,binding,'SliceTree');
 for(const record of binding.records)observer+=`\nimpl SliceTree for ${record.name} { fn visit(&self,v:&mut dyn FnMut(SliceToken)){v(SliceToken::ArrayStart);${record.fields.map(f=>`self.${f.rust}.visit(v);`).join('')}v(SliceToken::ArrayEnd);} }`;
 for(const owner of binding.customArrays)observer+=`\nimpl SliceTree for ${owner.name} { fn visit(&self,v:&mut dyn FnMut(SliceToken)){v(SliceToken::ArrayStart);for item in self.iter(){item.visit(v);}v(SliceToken::ArrayEnd);} }`;
 return observer;
}
