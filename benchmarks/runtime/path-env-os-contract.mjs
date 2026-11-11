import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import os from 'node:os';
import {containsCodeSnippet} from './source-contract-tokens.mjs';
export const pathEnvOsSpecs={
 path_parse_parts:{input:'reports/file.tar.gz',operations:1,weight:29,native:'const value=parse(input);',typescript:'const value=path.parse(input);',observation:'checksum+=value.root.length+value.dir.length+value.base.length+value.ext.length+value.name.length;'},
 path_format_options:{input:'reports',operations:1,weight:19,native:'const value=format(parts);',typescript:'const value=path.format(parts);',observation:'checksum+=value.length;'},
 path_resolve_parts:{input:'../next/file.txt',operations:1,weight:process.platform==='win32'?16:14,native:'const value=unwrap(resolve(root,input));',typescript:'const value=path.resolve(root,input);',observation:'checksum+=value.length;'},
 env_vars_snapshot:{operations:1,weight:42,native:'const rows=unwrap(vars());',typescript:"const rows=Object.entries(environment).map(([name,value])=>({name,value:value??''}));",observation:"if(row.name==='TSRUST_B03_TEXT'||row.name==='TSRUST_B03_EMPTY')"},
 env_get_snapshot:{operations:3,weight:12,native:'const value=unwrap(get(name));',typescript:'const value=environment[name];',observation:"const names=['TSRUST_B03_TEXT','TSRUST_B03_EMPTY','TSRUST_B03_MISSING'];"},
 os_parallelism_query:{operations:1,weight:1,native:'const value=unwrap(availableParallelism());',typescript:'const value=os.availableParallelism();',observation:"if(value<1){throw 'parallelism';}"},
 os_hostname_read:{operations:1,native:'const value=unwrap(hostname());',typescript:'const value=os.hostname();',observation:null},
};
export function pathEnvOsEnvironmentFor(row,base){
 if(row?.family!=='path-env-os')return base;
 const result={...base};
 for(const name of Object.keys(result))if(['TSRUST_B03_TEXT','TSRUST_B03_EMPTY','TSRUST_B03_MISSING'].includes(name.toUpperCase()))delete result[name];
 return {...result,TSRUST_B03_TEXT:'héllo 🦀',TSRUST_B03_EMPTY:''};
}
export function pathEnvOsChecksum(id,n,seed,input){
 const spec=pathEnvOsSpecs[id];assert(spec);assert.equal(input,spec.input);assert(Number.isSafeInteger(n)&&n>=0);assert(Number.isSafeInteger(seed));
 // Fixed lexical/fixture lengths. The independent hostname OS observation exposes
 // only byte length; no host or environment text is printed or retained.
 const weight=id==='os_hostname_read'?Buffer.byteLength(os.hostname(),'utf8'):spec.weight;
 const result=seed+n*weight;assert(Number.isSafeInteger(result));return result;
}
export function assertPathEnvOsSource(source,row,kind){
 const spec=pathEnvOsSpecs[row.id];assert(spec);assert.equal(row.family,'path-env-os');assert(['native','typescript'].includes(kind));
 assert.equal(createHash('sha256').update(source).digest('hex'),row[kind==='native'?'sourceSha256':'counterpartSha256']);
 assert.equal(row.iterations,2000);assert.equal(row.operationsPerIteration,spec.operations);assert.equal(row.inputText,spec.input);assert.equal(row.result,'int');assert.deepEqual(row.parameters,['int','int',...(spec.input===undefined?[]:['string'])]);
 for(const snippet of ['let checksum=seed;','while(i<n)','i++;','return checksum;',spec[kind],spec.observation].filter(Boolean))assert(containsCodeSnippet(source,snippet),row.id+': '+snippet);
 if(row.id==='path_parse_parts')for(const field of ['root','dir','base','ext','name'])assert(source.includes('value.'+field));
 if(row.id==='path_format_options')for(const snippet of ["name:'file.tar'","ext:'gz'",'String(input)','value.length!==19',"value.startsWith('reports')"])assert(containsCodeSnippet(source,snippet));
 if(row.id==='path_resolve_parts')assert(containsCodeSnippet(source,"value.endsWith('file.txt')"));
 if(row.id==='env_vars_snapshot')assert(containsCodeSnippet(source,'for(const row of rows)'));
 if(row.id==='env_vars_snapshot')assert(containsCodeSnippet(source,kind==='native'?'checksum+=row.name.length+row.value.length;':"checksum+=row.name.length+Buffer.byteLength(row.value,'utf8');"));
 if(row.id==='env_get_snapshot')assert(containsCodeSnippet(source,'for(const name of names)'));
 if(row.id==='env_get_snapshot')assert(containsCodeSnippet(source,kind==='native'?'checksum+=value===undefined?1:value.length;':"checksum+=value===undefined?1:Buffer.byteLength(value,'utf8');"));
 if(row.id==='os_hostname_read')assert(containsCodeSnippet(source,kind==='native'?'checksum+=value.length;':"checksum+=Buffer.byteLength(value,'utf8');"));
 if(row.id==='os_parallelism_query')assert(containsCodeSnippet(source,'checksum+=1;'));
}
