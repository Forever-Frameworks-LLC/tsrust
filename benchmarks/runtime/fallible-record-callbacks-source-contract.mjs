import {createHash} from 'node:crypto';
import {codeTokens} from './source-contract-tokens.mjs';
const spelling=text=>codeTokens(text,true).map(token=>token.text).join(' ');
const required=["return holder.run();", "typeof error === \"string\" ? error.length : typeof error === \"number\" ? error + 1000 : 0", "throw \"big\";", "run: (): int => base[0]", "throw \"x\".repeat(value);", "throw value;", "checked(plain, 2);", "outer = typeof error === \"string\" ? error.length : 0;", "[call(plain), guarded(plain), guarded(text), guarded(coded), outer]"];
export function assertFallibleRecordCallbacksSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='fallible_record_callbacks'||row.family!=='fallible-record-callbacks')throw Error('Fallible record callbacks family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Fallible record callbacks fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete fallible record callback scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Fallible record callbacks denominator');
 const tokens=spelling(source);
 for(const text of required)if(!tokens.includes(spelling(text)))throw Error('Fallible record callbacks scenario: '+text);
}
