import {createHash} from 'node:crypto';
import {codeTokens} from './source-contract-tokens.mjs';
const spelling=text=>codeTokens(text,true).map(token=>token.text).join(' ');
const required=["selected ||= value", "selected &&= value + 1", "label ||= \"guest\"", "label &&= \"member\"", "optional ||= \"present\"", "if (optional)", "while (remaining)", "!value ? 1 : 0", "!0 ? 1 : 0"];
export function assertTruthinessSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='truthiness'||row.family!=='truthiness')throw Error('Owned assignment values family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Owned assignment values fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete truthiness scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Owned assignment values denominator');
 const tokens=spelling(source);
 for(const text of required)if(!tokens.includes(spelling(text)))throw Error('Owned assignment values scenario: '+text);
}
