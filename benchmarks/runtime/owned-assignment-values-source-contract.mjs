import {createHash} from 'node:crypto';
import {codeTokens} from './source-contract-tokens.mjs';
const spelling=text=>codeTokens(text,true).map(token=>token.text).join(' ');
const required=["(text = \"x\".repeat(value)).length", "size(text = \"four\" + value)", "(values = [value, value + 1, value + 2]).length", "(point = {", "}).x", "(point.label = \"q\".repeat(value + 1)).length", "while ((current = rounds < value ? \"go\" : \"\") !== \"\") {", "[first, lent, count, values[2], coordinate, labelSize, point.label.length, rounds, text.length]"];
export function assertOwnedAssignmentValuesSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='owned_assignment_values'||row.family!=='owned-assignment-values')throw Error('Owned assignment values family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Owned assignment values fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete owned assignment value scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Owned assignment values denominator');
 const tokens=spelling(source);
 for(const text of required)if(!tokens.includes(spelling(text)))throw Error('Owned assignment values scenario: '+text);
}
