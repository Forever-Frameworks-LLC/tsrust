import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
export function assertSetAlgebraSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='set_algebra'||row.family!=='set-algebra')throw Error('Set algebra family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Set algebra fingerprint');
 if(kind==='typescript'&&!source.startsWith('/// <reference lib="esnext.collection" />\n'))throw Error('Set algebra declaration library');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete set algebra scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Set algebra denominator');
 for(const text of ['left.union(right)','left.intersection(right)','left.difference(right)','left.symmetricDifference(right)','left.intersection(small)','const copied=names.union(more);names.clear();','copied.forEach(','text=text+value+"|"','return output;'])if(!containsCodeSnippet(source, text))throw Error('Set algebra scenario: '+text);
}
