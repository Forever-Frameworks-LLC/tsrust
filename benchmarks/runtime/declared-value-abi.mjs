import {sourceIdentityMatches} from './format-source-identity.mjs';
// SOURCE-ONLY proposal. Call on exact future CLI emission before rustc/adapters.
// No Record0, field1, container or forwarding-closure identity is assumed.
import crypto from 'node:crypto';
import {assertDeclaredPolicy} from './declared-value-observation.mjs';
const hash=text=>crypto.createHash('sha256').update(text).digest('hex'),compact=text=>text.replace(/\s+/g,'');
const fail=id=>{throw Error('Declared-function benchmark source/native ABI changed: '+id);};
function split(text){const result=[];let level=0,start=0;for(let i=0;i<text.length;i++){if('<(['.includes(text[i]))level++;if('>)]'.includes(text[i]))level--;if(level<0)throw Error('Unbalanced native type');if(text[i]===','&&level===0){result.push(text.slice(start,i));start=i+1;}}if(level)throw Error('Unbalanced native type');result.push(text.slice(start));return result.map(compact).filter(Boolean);}
function body(text,open){let depth=1;for(let i=open+1;i<text.length;i++){if(text[i]==='{')depth++;if(text[i]==='}'&&--depth===0)return text.slice(open+1,i);}throw Error('Unbalanced reviewed native declaration');}
function signature(text,name){const pattern=new RegExp('fn\\s+'+name+'\\s*\\(([^)]*)\\)\\s*->\\s*([^\\n{]+)\\s*\\{','g'),matches=[...text.matchAll(pattern)];if(matches.length!==1)throw Error('Native function signature not unique: '+name);return{parameters:split(matches[0][1]).map(p=>{const m=/^(?:mut)?(value\d+):(.+)$/.exec(p);if(!m)throw Error('Unexpected native argument');return{name:m[1],type:m[2]};}),result:compact(matches[0][2])};}
export function assertDeclaredSource(source,row){assertDeclaredPolicy(row);if(typeof source!=='string'||!sourceIdentityMatches(row.declaredSourceSha256,hash(source)))fail(row.id);
 const signatures=[...source.matchAll(/function\s+workload\s*\(([^)]*)\)\s*:\s*([A-Za-z0-9_\[\]\s]+)\s*\{/g)];
 const wanted=row.parameters.length===2?'n:int,seed:int':'n:int,seed:int,input:string';if(signatures.length!==1||compact(signatures[0][1])!==wanted||compact(signatures[0][2])!==row.result)fail(row.id);
 if(JSON.stringify(row.parameters)!==JSON.stringify(row.inputText===undefined?['int','int']:['int','int','string']))fail(row.id);
 const schemas=new Map();for(const m of source.matchAll(/type\s+(\w+)\s*=\s*\{([^{}]*)\}\s*;/g)){if(schemas.has(m[1]))fail(row.id);const fields=m[2].split(';').map(compact).filter(Boolean).map(f=>{const n=/^(\w+):([\w\[\]]+)$/.exec(f);if(!n)fail(row.id);return[n[1],n[2]];});schemas.set(m[1],fields);}
 if(JSON.stringify([...schemas])!==JSON.stringify(Object.entries(row.recordSchemas??{})))fail(row.id);
 const names=[...source.matchAll(/\bfunction\s+(\w+)\s*\(/g)].map(m=>m[1]);if(names[0]!=='workload'||new Set(names).size!==names.length||names.at(-1)!=='main')fail(row.id);
 for(const target of row.declaredTargets)if(!names.includes(target.name)||target.name==='main'||target.name==='workload')fail(row.id);return names;
}
function verifyDeclaredBinding(emitted,row,source,reference){const names=assertDeclaredSource(source,row);if(!emitted.includes('fn main()'))fail(row.id);
 const fn=signature(emitted,'function0'),wanted=['i64','i64',...(row.inputText===undefined?[]:['&String'])];if(JSON.stringify(fn.parameters.map(p=>p.type))!==JSON.stringify(wanted))fail(row.id);
 const aliases=new Map(),custom=new Map(),records=new Map(),bindings=new Map(),containers=new Set();
 for(const m of emitted.matchAll(/type\s+(RecordArray\d+)\s*=\s*([^;]+);/g)){if(aliases.has(m[1]))fail(row.id);aliases.set(m[1],compact(m[2]));}
 for(const m of emitted.matchAll(/struct\s+(RecordArray\d+)\s*\(\s*Vec\s*<\s*(Record\d+)\s*>\s*\)\s*;/g)){if(custom.has(m[1])||aliases.has(m[1]))fail(row.id);custom.set(m[1],m[2]);}
 for(const m of emitted.matchAll(/struct\s+(Record\d+)\s*\{([^{}]*)\}/g)){if(records.has(m[1]))fail(row.id);records.set(m[1],split(m[2]).map(f=>{const n=/^(field\d+):(.+)$/.exec(f);if(!n)fail(row.id);return{name:n[1],type:n[2]};}));}
 function check(rust,type,active=new Set()){
  if(type.endsWith('[]')){let element,owner=rust;if(aliases.has(rust)){owner=aliases.get(rust);}if(custom.has(rust)){element=custom.get(rust);containers.add(rust);}else{const m=/^(Vec|Array|NumericArray)<(.+)>$/.exec(owner);if(!m)fail(row.id);element=m[2];containers.add(m[1]);if(m[1]!=='Vec'&&!new RegExp('\\b(?:enum|struct)\\s+'+m[1]+'\\s*<').test(emitted))fail(row.id);}check(element,type.slice(0,-2),active);return;}
  if(Object.hasOwn(row.recordSchemas??{},type)){const fields=records.get(rust),schema=row.recordSchemas[type];if(!fields||fields.length!==schema.length||fields.some((f,i)=>f.name!=='field'+i))fail(row.id);if(bindings.has(type)&&bindings.get(type).name!==rust)fail(row.id);bindings.set(type,{name:rust,fields:fields.map((f,i)=>({...f,source:schema[i][0],declaredType:schema[i][1]}))});const key=rust+':'+type;if(active.has(key))return;const next=new Set(active).add(key);for(let i=0;i<fields.length;i++)check(fields[i].type,schema[i][1],next);return;}
  if(rust!==({int:'i64',number:'f64',boolean:'bool',string:'String'})[type])fail(row.id);if(type==='string'&&!/type\s+String\s*=\s*std::borrow::Cow\s*<\s*'static\s*,\s*str\s*>\s*;/.test(emitted))fail(row.id);
 }
 check(fn.result,row.result);
 const closureStructs=new Map();for(const m of emitted.matchAll(/struct\s+(Closure\d+)\s*\{([^{}]*)\}/g))closureStructs.set(m[1],split(m[2]));
 function calls(name){const pattern=new RegExp('impl\\s+'+name+'\\s*\\{\\s*fn\\s+call\\s*\\(([^)]*)\\)\\s*->\\s*([^\\n{]+)\\s*\\{','g');return[...emitted.matchAll(pattern)].map(m=>({parameters:split(m[1]),result:compact(m[2]),body:compact(body(emitted,m.index+m[0].lastIndexOf('{')))}));}
 const targets=[];
 for(const target of (reference?[]:row.declaredTargets)){const id=names.indexOf(target.name),actual=signature(emitted,'function'+id);if(actual.parameters.length!==target.params.length)fail(row.id);
  for(let i=0;i<target.params.length;i++){const [mode,type]=target.params[i],native=actual.parameters[i].type;if(mode==='read'){if(!native.startsWith('&')||native.startsWith('&mut'))fail(row.id);check(native.slice(1),type);}else if(mode==='mutate'){if(!native.startsWith('&mut'))fail(row.id);check(native.slice(4),type);}else check(native,type);}
  let returnedClosure=null;if(typeof target.result==='string')check(actual.result,target.result);else{const expected=target.result.callable,fields=closureStructs.get(actual.result);if(!fields||fields.length!==expected.captureTypes.length)fail(row.id);for(let i=0;i<fields.length;i++){const m=/^capture\d+:(.+)$/.exec(fields[i]);if(!m)fail(row.id);check(m[1],expected.captureTypes[i]);}const methods=calls(actual.result);if(methods.length!==1||methods[0].parameters[0]!=='&self'||methods[0].parameters.length!==expected.params.length+1)fail(row.id);for(let i=0;i<expected.params.length;i++){const m=/^value\d+:(.+)$/.exec(methods[0].parameters[i+1]);if(!m)fail(row.id);const [mode,type]=expected.params[i];if(mode==='read'){if(!m[1].startsWith('&')||m[1].startsWith('&mut'))fail(row.id);check(m[1].slice(1),type);}else check(m[1],type);}check(methods[0].result,expected.result);returnedClosure=actual.result;}
  const wrappers=[];for(const [name,fields]of closureStructs){if(fields.length)continue;for(const method of calls(name)){const call='function'+id+'('+actual.parameters.map(p=>p.name).join(',')+')';if(method.body!==call)continue;if(method.parameters[0]!=='&self'||method.result!==actual.result||JSON.stringify(method.parameters.slice(1))!==JSON.stringify(actual.parameters.map(p=>p.name+':'+p.type)))fail(row.id);wrappers.push(name);}}
  if(wrappers.length!==1)fail(row.id);if(new RegExp('#\\[derive\\([^\\]]*(?:Copy|Clone)[^\\]]*\\)\\]\\s*struct\\s+'+wrappers[0]+'\\b').test(emitted)||new RegExp('impl\\s+(?:Copy|Clone)\\s+for\\s+'+wrappers[0]+'\\b').test(emitted))fail(row.id);targets.push({source:target.name,function:'function'+id,wrapper:wrappers[0],environment:'read',captureFields:0,parameterTypes:actual.parameters.map(p=>p.type),resultType:actual.result,returnedClosure});
 }
 return{protocol:reference?'hand-written-declared-result-abi-v1':'actual-emission-declared-function-abi-v1',id:row.id,sourceSha256:hash(source),emittedSha256:hash(emitted),resultType:fn.result,parameters:fn.parameters,records:Object.fromEntries(bindings),containers:[...containers].sort(),customRecordArrays:Object.fromEntries([...custom].filter(([name])=>containers.has(name))),targets};
}
// Compiler output retains the declared-target, wrapper and capture checks.
export function verifyDeclaredABI(emitted,row,source){return verifyDeclaredBinding(emitted,row,source,false);}
// Independent references bind the same source, owned result schema and actual fields.
// Their callback implementation uses ordinary Rust rather than compiler wrappers.
export function verifyDeclaredReferenceABI(emitted,row,source){return verifyDeclaredBinding(emitted,row,source,true);}
// Actual source/emission hash binding is mandatory before creating either adapter.
export function recordObserver(binding){let text='';for(const [source,record]of Object.entries(binding.records)){if(source!=='Row'&&source!=='Node')continue;if(record.fields.length!==3)throw Error('Separate three-field observer required');text+='impl SliceTree for '+record.name+' { fn visit(&self,v:&mut dyn FnMut(SliceToken)) { v(SliceToken::ArrayStart);'+record.fields.map(f=>'self.'+f.name+'.visit(v);').join('')+'v(SliceToken::ArrayEnd); } }\n';}for(const [owner,element]of Object.entries(binding.customRecordArrays))text+='impl SliceTree for '+owner+' { fn visit(&self,v:&mut dyn FnMut(SliceToken)) { v(SliceToken::ArrayStart);for value in self.iter(){value.visit(v);}v(SliceToken::ArrayEnd); } }\n';return text;}

export function pruneObserverContainers(observer,binding){for(const [name,header]of [['Array','impl<T:SliceTree> SliceTree for Array<T>'],['NumericArray','impl<T:SliceTree+Copy+Default> SliceTree for NumericArray<T>']]){if(binding.containers.includes(name))continue;const start=observer.indexOf(header);if(start<0||observer.indexOf(header,start+1)>=0)throw Error('Existing SliceTree observer container hook changed');const open=observer.indexOf('{',start);let depth=1,end=open+1;while(end<observer.length&&depth){if(observer[end]==='{')depth++;if(observer[end]==='}')depth--;end++;}if(depth)throw Error('Unbalanced SliceTree container hook');observer=observer.slice(0,start)+observer.slice(end);}return observer;}

export function assertDeclaredTSSource(source,row){assertDeclaredPolicy(row);if(typeof source!=='string'||!sourceIdentityMatches(row.declaredTSSha256,hash(source)))fail(row.id);}
