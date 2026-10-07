import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {dataUnionsIds} from './data-unions-oracle.mjs';
const ts=createRequire(import.meta.url)('typescript');
export function assertDataUnionsSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.family!=='data-unions'||!dataUnionsIds.includes(row.id))throw Error('Data unions family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Data unions fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete mixed data tag, clone, narrowing and mutable borrow scenario iteration'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Data unions denominator');
 for(const text of ['while (i < n)','return output;','output.push([','copyRow(source)','copied.owner=[base+5]','typeof value==="string"','value??"missing"','copiedText(source.owner)'])if(!containsCodeSnippet(source, text))throw Error('Actual data union scenario and full retention');
 if(!containsCodeSnippet(source, kind==='native'?'return clone(value);':'owner:typeof value.owner==="string"?value.owner.slice():value.owner.slice()'))throw Error('Independent counterpart owner copies');
 const file=ts.createSourceFile(row.id+'.ts',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);let unions=0,workload;
 function visit(node){if(ts.isUnionTypeNode(node))unions++;if(ts.isFunctionDeclaration(node)&&node.name?.text==='workload')workload=node;ts.forEachChild(node,visit);}
 visit(file);if(file.parseDiagnostics.length||!workload||workload.parameters.length!==3||!workload.type||unions!==4)throw Error('Actual typed data union operations');
}
