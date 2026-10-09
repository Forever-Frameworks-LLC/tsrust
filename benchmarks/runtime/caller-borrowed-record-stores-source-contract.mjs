import {createHash} from 'node:crypto';
import {containsCodeSnippet,findCodeSnippet} from './source-contract-tokens.mjs';
const required=[
 'lendMutable(holder);', 'scopedFirst = holder.run!();', 'scopedSecond = holder.run!();', 'scopedReleased = source[0];', 'scopedEffects = calls[0];', 'transferOnce(holder);', 'ownedFirst = holder.run!();', 'ownedSecond = holder.run!();',
 'installConditional(target, replacement, true, calls);', 'installLocalFirst(holder);', 'installLocalSecond(holder);', 'const localFirst = holder.run!();', 'return [localFirst, original[0], holder.run!(), calls[0]];', 'const localOwnerValues = inspectLocalOwners();',
 'keepFirst(holder);', 'keepSecond(holder);', 'installCapturedSource(target, original);', 'installCapturedSource(target, replacement);', 'independentFirst = holder.run!();', 'independentReleased = original[0];', 'independentReplacement = holder.run!();',
 'installCapturedSource(target, captured);', 'retainCaptured(holder);', 'retainedCapture = holder.run!();', 'clearCapturedSource(holder);', 'captured[0] += 1;', 'clearedCapture = captured[0];',
 'installConditional(target, source, true, calls);', 'calls[0] += offset;', 'performCaptured(holder, original);', 'performCaptured(holder, replacement);', 'capturedFirst = holder.run!();', 'capturedReleased = original[0];', 'capturedReplacement = holder.run!();', 'capturedEffects = calls[0];',
 'installConditional(target, source, plus, effects);', 'performBody(holder, original, true, calls);', 'performBody(holder, replacement, false, calls);', 'closureFirst = holder.run!();', 'closureReleased = original[0];', 'closureReplacement = holder.run!();', 'closureEffects = calls[0];',
 'const perform = installConditional;', 'perform(holder, original, true, calls);', 'perform(holder, replacement, false, calls);', 'forwardedFirst = holder.run!();', 'forwardedReleased = original[0];', 'forwardedReplacement = holder.run!();', 'forwardedEffects = calls[0];',
 'installConditional(holder, original, i % 2 === 0, calls);', 'installConditional(holder, replacement, i % 2 !== 0, calls);', 'conditionalFirst = holder.run!();', 'conditionalReleased = original[0];', 'conditionalReplacement = holder.run!();', 'conditionalEffects = calls[0];', 'calls[0] += 10;', 'const selected = chooseCallbackBranch(plus, calls) ? (calls[0] += 100, (): int => source[0] + 1) : (calls[0] += 200, (): int => source[0] + 2);',
 'calls[0] += selected();', 'holder.run = selected;',
 'installWithEffects(holder, original, effects);', 'installWithEffects(holder, replacement, effects);', 'factoryEffects = effects[0];', 'dataEffects = dataCalls[0];', 'calls[0] += 1;', 'installNullable(holder, action);', 'const original = counter(value);', 'const replacement = counter(value + 4);', 'factoryBefore = holder.run!();', 'factoryReleased = original();', 'factoryReplacement = holder.run!();', 'factoryAfter = replacement() + (holder.run === undefined ? 0 : 100);', 'installNullableData(holder, original, dataCalls);', 'installNullableData(holder, replacement, dataCalls);', 'original[0] += 1;', 'dataReleased = original[0];', 'dataReplacement = holder.run!();', 'dataAfter = replacement[0] + (holder.run === undefined ? 0 : 100);',
 'installNullable(holder, originalAction);', 'installNullable(holder, replacementAction);', 'nullableReleased = originalAction();', 'nullableReplacement = holder.run!();', 'nullableReplacementAfter = replacementAction() + (holder.run === undefined ? 0 : 100);',
 'installNullable(holder, action);', 'nullableBefore = holder.run!();', 'relayClearNullable(holder, i % 2 === 0);', 'nullableAfter = action();', 'nullableEmpty = holder.run === undefined ? 1 : 0;', 'holder.run = undefined;', 'if (early) { clearNullable(holder); return; }',
 'throw payload[0];', 'callbackFailure = guarded();', 'installDirect(holder, action);', 'installOnce(holder, once);', 'consumed = holder.run();', 'throw "installed";', 'if (fail) { throw "before"; }', 'afterFailure = holder.run() + (caught ? 0 : 100);', 'beforeFailure = holder.run() + (caught ? 0 : 100);', 'const alias = source;', 'holder.run = (): int => alias[0];',
 'installArray(holder, left);', 'installArray(holder, right);',
 'box.handler.run = (): int => left[0] + right[0];',
 'holder.run = action;', 'const alias = action;', 'holder.run = (): int => alias();',
 'holder.run = (): int => left() + right();',
 'source[0] += 1; return source[0];', '} base[0] += 1; {',
 'first = holder.run();', 'replaced = holder.run();',
 'nested = box.handler.run();', 'callback = callbackHolder.run();',
 'installDirect(callbackHolder, constant);', 'const readCallback = (): int => callbackHolder.run();', 'callbackReplaced = readCallback();',
 'installPair(pairHolder, constant, action);', 'pairReplaced = pairHolder.run();',
 'installMutableCallback(holder, secondAction);', 'const invoke = (): int => holder.run();', 'mutableReplaced = invoke();',
 'other[0] += 10; return other[0];',
 'paired = pairHolder.run();', 'changedFirst = holder.run();',
 'changedSecond = holder.run();', 'callbackFirst = holder.run();', 'callbackSecond = holder.run();', 'callbackAfter = action();', 'holder.run = (): int => action();',
 'output.push([first, replaced, nested, callback, paired, changedFirst, changedSecond, callbackFirst, callbackSecond, callbackAfter, base[0], callbackReplaced, pairReplaced, mutableReplaced, consumed, afterFailure, beforeFailure, callbackFailure, nullableBefore, nullableAfter, nullableEmpty, nullableReleased, nullableReplacement, nullableReplacementAfter, factoryBefore, factoryReleased, factoryReplacement, factoryAfter, dataReleased, dataReplacement, dataAfter, factoryEffects, dataEffects, conditionalFirst, conditionalReleased, conditionalReplacement, conditionalEffects, forwardedFirst, forwardedReleased, forwardedReplacement, forwardedEffects, closureFirst, closureReleased, closureReplacement, closureEffects, capturedFirst, capturedReleased, capturedReplacement, capturedEffects, retainedCapture, clearedCapture, independentFirst, independentReleased, independentReplacement, localOwnerValues[0], localOwnerValues[1], localOwnerValues[2], localOwnerValues[3], scopedFirst, scopedSecond, scopedReleased, scopedEffects, ownedFirst, ownedSecond]);'
];
export function assertCallerBorrowedRecordStoresSource(source,row,kind) {
 if(findCodeSnippet(source,'lendMutable(holder);').length!==2)throw Error('Scoped mutable installer invocation count');
 if(findCodeSnippet(source,'holder.run = undefined;').length!==3)throw Error('Caller-borrowed clear helper count');
 if(findCodeSnippet(source,'calls[0] += 1;').length!==4)throw Error('Caller-borrowed installer effect count');
 if(findCodeSnippet(source,'holder.run = action;').length!==5)throw Error('Caller-borrowed direct store count');
 if(!['native','typescript'].includes(kind)||row.id!=='caller_borrowed_record_stores'||row.family!=='caller-borrowed-record-stores')throw Error('Caller-borrowed record store family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Caller-borrowed record store fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete caller-borrowed input scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Caller-borrowed record store denominator');
 for(const text of required)if(!containsCodeSnippet(source,text))throw Error('Caller-borrowed record store scenario: '+text);
}
