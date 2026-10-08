import {createHash} from 'node:crypto';
import {codeTokens} from './source-contract-tokens.mjs';
const spelling=text=>codeTokens(text,true).map(token=>token.text).join(' ');
const required=['const factorial = function fact(count: int): int {','count * fact(count - 1)','const walk = function down(depth: int): void {','total += depth + base[0];','down(depth - 1);','walk(3);','[1, 2].map(function shift(item: int): int {','return item + base[1];','const make = (offset: int) => function add(count: int): int {','1 + add(count - 1)','const pick = function choose<T>(left: T, right: T, first: boolean): T {','[factorial(value), total, mapped[0], mapped[1], make(value)(2), pick(value, 0, true), pick("ab", "c", false).length]','return output;'];
export function assertNamedFunctionExpressionsSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='named_function_expressions'||row.family!=='named-function-expressions')throw Error('Named function expression family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Named function expression fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete named function expression scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Named function expression denominator');
 const tokens=spelling(source);
 for(const text of required)if(!tokens.includes(spelling(text)))throw Error('Named function expression scenario: '+text);
}
