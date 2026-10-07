import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {objectKeysIds} from './object-keys-oracle.mjs';
const ts=createRequire(import.meta.url)('typescript');
export function assertObjectKeysSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.family!=='object-keys'||!objectKeysIds.includes(row.id))throw Error('Object keys family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Object keys fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete record order, copied keys and UTF-16 key scenario iteration'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Object keys denominator');
 for(const text of ['while (i < n)','return output;','output.push([','Object.keys(source)','Object.keys(copied)','Object.keys(input)','names[0]="changed"','{gamma:true,alpha:base,beta:input.slice()}','{beta:input.slice(),gamma:false,alpha:base}'])if(!containsCodeSnippet(source, text))throw Error('Actual Object keys scenario');
 if(!containsCodeSnippet(source, kind==='native'?'clone(source)':'{...source}'))throw Error('Actual record clone');
 const file=ts.createSourceFile(row.id+'.ts',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);let keys=0,workload;
 function visit(node){if(ts.isCallExpression(node)&&ts.isPropertyAccessExpression(node.expression)&&node.expression.expression.getText(file)==='Object'&&node.expression.name.text==='keys')keys++;if(ts.isFunctionDeclaration(node)&&node.name?.text==='workload')workload=node;ts.forEachChild(node,visit);}
 visit(file);if(file.parseDiagnostics.length||!workload||workload.parameters.length!==3||!workload.type||keys!==3)throw Error('Typed Object keys calls');
}
