import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
export function assertSetsSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='set_values'||row.family!=='sets')throw Error('Set family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Set fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete set scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Set denominator');
 const integer=kind==='native'?'int':'number';
 for(const text of [`function workload(n: ${integer}, seed: ${integer}, input: string): string[][]`,`new Set<${integer}>([i+seed,i+seed+1,i+seed])`,'values.has(i+seed)','values.delete(i+seed)','values.add(i+seed)','keys.join(",")','new Set<string>([String(input),"tail",String(input)])','for(const key of values)', 'const copied=new Set(labels);', 'labels.clear();', 'labelKeys.join("|")', 'labels.keys()', 'return output;'])if(!containsCodeSnippet(source, text))throw Error('Set scenario: '+text);
}
