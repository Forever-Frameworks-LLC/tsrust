import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {tupleTypesIds} from './tuple-types-oracle.mjs';
const ts=createRequire(import.meta.url)('typescript');
export function assertTupleTypesSource(source,row,kind) {
 if(!['native','typescript'].includes(kind)||row.family!=='tuple-types'||!tupleTypesIds.includes(row.id))throw Error('Tuple types source kind/family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Tuple types source fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete tuple construction, copy, mutation and map scenario iteration')throw Error('Tuple types scenario denominator');
 if(row.resultEncoding!=='array-shape-scalar-v1')throw Error('Tuple types observer protocol');
 for(const text of ['while (i < n)','return output;','output.push([','pair[0]=base+1;','pair[1]=input.slice();','rows.map(','copyPair(pair)'])if(!containsCodeSnippet(source, text))throw Error('Actual tuple operations and full retention');
 const file=ts.createSourceFile(row.id+'.ts',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);let tuples=0,workload;
 function visit(node){if(ts.isTupleTypeNode(node))tuples++;if(ts.isFunctionDeclaration(node)&&node.name?.text==='workload')workload=node;ts.forEachChild(node,visit);}
 visit(file);
 if(file.parseDiagnostics.length||!workload||workload.parameters.length!==3||!workload.type||tuples!==6)throw Error('Actual typed tuple scenario');
}
