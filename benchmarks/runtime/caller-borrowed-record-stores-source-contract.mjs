import {createHash} from 'node:crypto';
import {containsCodeSnippet} from './source-contract-tokens.mjs';
const required=[
 'const alias = source;', 'holder.run = (): int => alias[0];',
 'installArray(holder, left);', 'installArray(holder, right);',
 'box.handler.run = (): int => left[0] + right[0];',
 'const alias = action;', 'holder.run = (): int => alias();',
 'holder.run = (): int => left() + right();',
 'source[0] += 1; return source[0];', '} base[0] += 1; {',
 'first = holder.run();', 'replaced = holder.run();',
 'nested = box.handler.run();', 'callback = callbackHolder.run();',
 'installCallback(callbackHolder, constant);', 'const readCallback = (): int => callbackHolder.run();', 'callbackReplaced = readCallback();',
 'installPair(pairHolder, constant, action);', 'pairReplaced = pairHolder.run();',
 'installMutableCallback(holder, secondAction);', 'const invoke = (): int => holder.run();', 'mutableReplaced = invoke();',
 'other[0] += 10; return other[0];',
 'paired = pairHolder.run();', 'changedFirst = holder.run();',
 'changedSecond = holder.run();', 'callbackFirst = holder.run();', 'callbackSecond = holder.run();', 'callbackAfter = action();', 'holder.run = (): int => action();',
 'output.push([first, replaced, nested, callback, paired, changedFirst, changedSecond, callbackFirst, callbackSecond, callbackAfter, base[0], callbackReplaced, pairReplaced, mutableReplaced]);'
];
export function assertCallerBorrowedRecordStoresSource(source,row,kind) {
 if(!['native','typescript'].includes(kind)||row.id!=='caller_borrowed_record_stores'||row.family!=='caller-borrowed-record-stores')throw Error('Caller-borrowed record store family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Caller-borrowed record store fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete caller-borrowed input scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Caller-borrowed record store denominator');
 for(const text of required)if(!containsCodeSnippet(source,text))throw Error('Caller-borrowed record store scenario: '+text);
}
