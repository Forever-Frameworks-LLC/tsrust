import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
const ts=createRequire(import.meta.url)('typescript');
export function assertGlobalNumberNamesSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.family!=='global-number-names'||row.id!=='global_number_names')throw Error('Global number name family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Global number name fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete global-number-name scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Global number name denominator');
 for(const text of ['while (i < n)','const whole=parseInt(String(base)+"px");','const hex=parseInt(String(base),16);','const part=parseFloat(String(base)+".25x");','const missing=parseInt(input);','let lowest=Infinity;','if(part<lowest)lowest=part;','const picked=isNaN(missing)?NaN:missing+1.0;','output.push([String(whole+hex),String(part),String(isNaN(missing)),String(isFinite(lowest)&&!isFinite(-Infinity)),String(picked)]);','return output;'])if(!source.includes(text))throw Error('Actual global-number-name scenario: '+text);
 if(source.includes('Number.'))throw Error('Number member in a global-number-name scenario');
 if(kind==='typescript'){
  const file=ts.createSourceFile(row.id+'.ts',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);const counts={};
  (function visit(node){
   if(ts.isCallExpression(node)&&ts.isIdentifier(node.expression)&&['parseInt','parseFloat','isNaN','isFinite'].includes(node.expression.text))counts[node.expression.text]=(counts[node.expression.text]??0)+1;
   if(ts.isIdentifier(node)&&['NaN','Infinity'].includes(node.text))counts[node.text]=(counts[node.text]??0)+1;
   ts.forEachChild(node,visit);})(file);
  if(file.parseDiagnostics.length||JSON.stringify(counts)!==JSON.stringify({parseInt:3,parseFloat:1,Infinity:2,isNaN:2,NaN:1,isFinite:2}))throw Error('Actual global number names: '+JSON.stringify(counts));
 }
}
