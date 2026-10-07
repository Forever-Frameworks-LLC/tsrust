import {containsCodeSnippet, findCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
const ts=createRequire(import.meta.url)('typescript');
export function assertSwitchStatementsSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.family!=='switch-statements'||row.id!=='switch_statements')throw Error('Switch family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Switch fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete switch scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Switch denominator');
 for(const text of ['switch(code%7){','case 1:\n  case 2:return "low";','while (i < n)','switch((base+k)%3){','case 0:continue;','default:steps+=1;','switch(word){\n   default:text="number";break;','switch(mixed){\n   case "three":seen=mixed.toUpperCase();break;','output.push([tier(base),String(steps),text,seen]);','return output;'])if(!containsCodeSnippet(source, text))throw Error('Actual switch scenario: '+text);
 if(findCodeSnippet(source, 'switch(').length!==4)throw Error('Four actual switch statements');
 if(kind==='typescript'){
  const file=ts.createSourceFile(row.id+'.ts',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);let switches=0,clauses=0;
  (function visit(node){if(ts.isSwitchStatement(node))switches++;if(ts.isCaseClause(node)||ts.isDefaultClause(node))clauses++;ts.forEachChild(node,visit);})(file);
  if(file.parseDiagnostics.length||switches!==4||clauses!==14)throw Error('Actual typed switch statements');
 }
}
