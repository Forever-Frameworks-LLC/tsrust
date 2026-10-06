import {assertGenericAdapterSource} from './generic-value-adapters-source-contract.mjs';
import {assertGenericResultSource} from './generic-function-results-source-contract.mjs';
import {assertCapturedGenericSource} from './generic-function-captures-source-contract.mjs';
import {assertDeferredGenericSource} from './generic-function-deferred-source-contract.mjs';
import {assertGenericContextSource} from './generic-function-contexts-source-contract.mjs';
import {assertNestedRestSource} from './nested-function-rest-source-contract.mjs';
import {assertNestedDefaultSource} from './nested-function-defaults-source-contract.mjs';
import {assertNestedResultSource} from './nested-function-results-source-contract.mjs';
import {assertNestedOwnedSource} from './nested-function-owned-escape-source-contract.mjs';
import {assertNestedFunctionDeclarationsSource} from './nested-function-declarations-source-contract.mjs';
import {assertNestedFunctionCapturesSource} from './nested-function-captures-source-contract.mjs';
import {assertNumericStringConversionSource} from './numeric-string-conversion-source-contract.mjs';
import {assertScalarUnaryPlusSource} from './scalar-unary-plus-source-contract.mjs';
import {assertStringConcatenationSource} from './string-concatenation-source-contract.mjs';
import {assertCommaContextsSource} from './comma-contexts-source-contract.mjs';
import {assertCommaExpressionsSource} from './comma-expressions-source-contract.mjs';
import {assertStringIndexingSource} from './string-indexing-source-contract.mjs';
import {assertOptionalFieldsSource} from './optional-fields-source-contract.mjs';
import {assertInterfacesSource} from './interfaces-source-contract.mjs';
import {assertTypeAliasesSource} from './type-aliases-source-contract.mjs';
import {assertNumericLiteralsSource} from './numeric-literals-source-contract.mjs';
import {assertMapGroupBySource} from './map-group-by-source-contract.mjs';
import {assertIterableCallSpreadSource} from './iterable-call-spread-source-contract.mjs';
import {assertIterableArraySpreadSource} from './iterable-array-spread-source-contract.mjs';
import {assertArrayFromCollectionsSource} from './array-from-collections-source-contract.mjs';
import {assertStringIterationSource} from './string-iteration-source-contract.mjs';
import {assertMapIterationSource} from './map-iteration-source-contract.mjs';
import {assertSetAlgebraSource} from './set-algebra-source-contract.mjs';
import {assertSetRelationsSource} from './set-relations-source-contract.mjs';
import {assertMapForEachSource} from './map-for-each-source-contract.mjs';
import {assertArrayCallbackErrorsSource} from './array-callback-errors-source-contract.mjs';
import {assertSetForEachSource} from './set-for-each-source-contract.mjs';
import {assertSetEntriesSource} from './set-entries-source-contract.mjs';
import {assertMapSnapshotsSource} from './map-snapshots-source-contract.mjs';
import {assertMapsSource} from './maps-source-contract.mjs';
import {assertSetsSource} from './sets-source-contract.mjs';
import {assertEnumsSource} from './enums-source-contract.mjs';
import {assertTypeAssertionsSource} from './type-assertions-source-contract.mjs';
import {assertExceptionCompletionSource} from './exception-completion-source-contract.mjs';
import {assertCallableExpressionsSource} from './callable-expressions-source-contract.mjs';
import {assertLabeledControlSource} from './labeled-control-source-contract.mjs';
import {assertInOperatorSource} from './in-operator-source-contract.mjs';
import {assertDeferredInitializationSource} from './deferred-initialization-source-contract.mjs';
import {assertGlobalNumberNamesSource} from './global-number-names-source-contract.mjs';
import {assertFunctionExpressionsSource} from './function-expressions-source-contract.mjs';
import {assertSwitchStatementsSource} from './switch-statements-source-contract.mjs';
import {assertBracelessBodiesSource} from './braceless-bodies-source-contract.mjs';
import {assertDestructuringAssignmentSource} from './destructuring-assignment-source-contract.mjs';
import {assertDeclarationListsSource} from './declaration-lists-source-contract.mjs';
import {assertForPatternInitializersSource} from './for-pattern-initializers-source-contract.mjs';
import {assertNestedRestPatternsSource} from './nested-rest-patterns-source-contract.mjs';
import {assertDestructuringRestSource} from './destructuring-rest-source-contract.mjs';
import {assertObjectHasOwnSource} from './object-has-own-source-contract.mjs';
import {compilerSourceInputs} from './compiler-source-inputs.mjs';
import {assertTestingAssertionSource} from './testing-assertions-source-contract.mjs';
import {assertDestructuringDefaultsSource} from './destructuring-defaults-source-contract.mjs';
import {assertBorrowedOptionalFallbackSource} from './borrowed-optional-fallback-source-contract.mjs';
import {assertDataUnionsSource} from './data-unions-source-contract.mjs';
import {assertUnionArraySource} from './union-array-elements-source-contract.mjs';
import {assertObjectValuesSource} from './object-values-entries-source-contract.mjs';
import {assertDestructuringSource} from './destructuring-source-contract.mjs';
import {assertObjectKeysSource} from './object-keys-source-contract.mjs';
import {assertArrayEntriesSource} from './array-entries-source-contract.mjs';
import {assertTupleTypesSource} from './tuple-types-source-contract.mjs';
import {assertStringNormalizeSource} from './string-normalize-source-contract.mjs';
import {assertArrayLocaleStringSource} from './array-locale-string-source-contract.mjs';
import {assertNumberLocaleStringSource} from './number-locale-string-source-contract.mjs';
import {assertStringRawSource} from './string-raw-source-contract.mjs';
import {assertArrayFromSource} from './array-from-source-contract.mjs';
import {assertStringAtSource} from './string-at-source-contract.mjs';
import {assertNumberParseIntSource} from './number-parse-int-source-contract.mjs';
import {assertNumberToStringSource} from './number-to-string-source-contract.mjs';
import {assertObjectIsSource} from './object-is-source-contract.mjs';
import {assertMathF16roundSource} from './math-f16round-source-contract.mjs';
import {assertMathRandomSource} from './math-random-source-contract.mjs';
import {assertStringFromCodesSource} from './string-from-codes-source-contract.mjs';
import {assertStringCodeAtSource} from './string-code-at-source-contract.mjs';
import {assertStringLocaleCaseSource} from './string-locale-case-source-contract.mjs';
import {assertArrayKeysValuesSource} from './array-keys-values-source-contract.mjs';
import {assertArrayLiteralSpreadSource} from './array-literal-spread-source-contract.mjs';
import {assertCallSpreadSource} from './call-spread-source-contract.mjs';
import {assertRestParameterSource} from './rest-parameters-source-contract.mjs';
import {assertOptionalDefaultParameterSource} from './optional-default-parameters-source-contract.mjs';
import {assertNullableCallableSource} from './nullable-callable-source-contract.mjs';
import {assertOptionalChainSource} from './optional-chain-source-contract.mjs';
import {assertNullishAssignmentSource} from './nullish-assignment-source-contract.mjs';
import {assertNullishSource} from './nullish-source-contract.mjs';
import {assertOfSource} from './array-of-source-contract.mjs';
import {assertFlatSource} from './array-flat-source-contract.mjs';
import {assertSortingSource} from './array-sorting-source-contract.mjs';
import {assertSpliceSource} from './array-splice-source-contract.mjs';
import {assertUnshiftSource} from './array-unshift-source-contract.mjs';
import {assertFindValueSource} from './array-find-values-source-contract.mjs';
import {assertArrayAtSource} from './array-at-source-contract.mjs';
import {assertArrayRemovalSource} from './array-removal-source-contract.mjs';
import {assertNarrowingSource} from './narrowing-source-contract.mjs';
import {assertNullableTransportSource} from './nullable-transport-source-contract.mjs';
import {assertArrayToStringSource} from './array-to-string-source-contract.mjs';
import {assertArrayJoinSource} from './array-join-source-contract.mjs';
import {assertValueTypesSource} from './value-types-source-contract.mjs';
import {assertHtmlStringSource} from './html-string-source-contract.mjs';
import {assertStringSubstrSource} from './string-substr-source-contract.mjs';
import {assertSplitOwnershipSource} from './split-ownership-source-contract.mjs';
import {assertTemplateStringSource} from './template-string-source-contract.mjs';
import {assertQuotedStringLiteralSource} from './quoted-string-literals-source-contract.mjs';
import {assertStringWellFormedSource} from './string-well-formed-source-contract.mjs';
import {assertNumberParseFloatSource} from './number-parse-float-source-contract.mjs';
import {assertNumberFormatSource} from './number-format-source-contract.mjs';
import {assertIntToStringSource} from './int-to-string-source-contract.mjs';
import {assertPrimitiveIdentitySource} from './primitive-identity-source-contract.mjs';
import {assertStringSubstringSource} from './string-substring-source-contract.mjs';
import {assertStringConcatPaddingSource} from './string-concat-padding-source-contract.mjs';
import {assertConditionalSource} from './conditional-source-contract.mjs';
import {isCopying,assertCopyingPolicy,validateCopyingObservation} from './copying-array-observation.mjs';
import {assertCopyingSource,assertCopyingTSSource} from './copying-array-result-abi.mjs';
import {isRange,assertRangePolicy,validateRangeObservation} from './range-mutation-observation.mjs';
import {assertRangeSource,assertRangeTSSource} from './range-mutation-result-abi.mjs';
import {isHigher,validateHigherObservation} from './higher-order-observation.mjs';
import {assertHigherSource,assertHigherTSSource} from './higher-order-result-abi.mjs';
import {assertDeclaredSource,assertDeclaredTSSource} from './declared-value-abi.mjs';
import {isDeclared,assertDeclaredPolicy,validateDeclaredObservation} from './declared-value-observation.mjs';
import {isConcat,validateConcatObservation} from './concat-observation.mjs';
import {isApproximate,loadApproximateReference,assertApproximateCases,assertApproximateArtifactsUnchanged,approximatePointProbeSources,approximateAdapterSources} from './approximate-harness.mjs';
import {validateApproximatePointProbe,validateApproximateMeasurement} from './approximate-scalar.mjs';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import crypto from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {performance} from 'node:perf_hooks';
import {fileURLToPath} from 'node:url';
import {expectedChecksum, expectedExactResult} from './oracle.mjs';
import {requiresExactResult,validateExactObservation} from './result-observation.mjs';
import {measurementPlan} from './measurement-plan.mjs';
import {assertConcatSource,assertConcatTSSource} from './concat-abi.mjs';
import {assertReductionSource} from './reduction-abi.mjs';
import {isReduction} from './reduction-observation.mjs';
import {assertABI, assertForEachSource, adaptRustDriver, adaptTSDriver, assertCLIInputs, numberAdapterSources, sliceAdapterSources, reductionAdapterSources, concatAdapterSources, declaredAdapterSources, higherAdapterSources, rangeAdapterSources, copyingAdapterSources} from './adapters.mjs';
const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '../..');
const build = path.join(root, 'target/runtime-comparisons');
fs.mkdirSync(build, {recursive: true});
const manifest = JSON.parse(fs.readFileSync(path.join(here, 'manifest.json')));
const pins = JSON.parse(fs.readFileSync(path.join(here, 'versions.json')));
const plan = measurementPlan(process.argv, manifest);
const verify = plan.verify;
const select = process.argv.find(a => a.startsWith('--only='))?.slice(7);
const selections = select?.split(',');
const workloads = manifest.workloads.filter(w => !selections || selections.includes(w.id));
if (!workloads.length || selections?.some(id => !manifest.workloads.some(w => w.id === id))) throw new Error(`Unknown workload: ${select}`);
assertCLIInputs(workloads);
for(const w of workloads)isApproximate(w); // Fail closed before preparing unknown protocols.
const env = {...process.env, PATH: path.dirname(process.execPath) + path.delimiter + process.env.PATH, DENO_DIR: path.join(build, 'deno-cache'), XDG_CACHE_HOME: path.join(build, 'cache'), TMPDIR: build};
// Ambient loaders and ts-node options must not silently change the comparison.
for (const name of Object.keys(env)) if (name === 'NODE_OPTIONS' || name.startsWith('TS_NODE_')) delete env[name];
const tsc = [process.execPath, path.join(here, 'node_modules/typescript/bin/tsc')];
const tsx = [process.execPath, path.join(here, 'node_modules/tsx/dist/cli.mjs')];
const tsnode = [process.execPath, path.join(here, 'node_modules/ts-node/dist/bin.js')];
const deno = path.join(here, 'node_modules/.bin/deno');
const compiler = path.join(root, 'target/release/tsrust');
const rustc = ['rustup', 'run', pins.rust, 'rustc'];
function run(command, args = []) {
 const start = performance.now();
 const child = spawnSync(command[0], [...command.slice(1), ...args.map(String)], {cwd: here, env, encoding: 'utf8', timeout: 120000, maxBuffer: 16 * 1024 * 1024});
 const ms = performance.now() - start;
 if (child.error || child.status !== 0) throw new Error(`${command.join(' ')} failed: ${child.error ?? (child.stderr + child.stdout)}`);
 return {ms, stdout: child.stdout.trim()};
}
const versions = {
 node: process.version,
 bun: run(['bun'], ['--version']).stdout,
 rust: run(rustc, ['--version']).stdout,
 typescript: JSON.parse(fs.readFileSync(path.join(here, 'node_modules/typescript/package.json'))).version,
 tsx: JSON.parse(fs.readFileSync(path.join(here, 'node_modules/tsx/package.json'))).version,
 'ts-node': run(tsnode, ['--version']).stdout.replace(/^v/, ''),
 deno: run([deno], ['--version']).stdout
};
for (const [tool, pinned] of Object.entries(pins)) {
 const actual = tool === 'rust' ? versions.rust.split(' ')[1] : tool === 'deno' ? versions.deno.split(' ')[1] : versions[tool];
 if (actual !== pinned) throw new Error(`${tool} version ${actual}; expected ${pinned}. Update pins deliberately and record a new baseline.`);
}
run(['cargo'], ['build', '--release', '--offline', '--quiet', '--manifest-path', path.join(root, 'Cargo.toml')]);
const hash = content => crypto.createHash('sha256').update(content).digest('hex');
const driverTS = fs.readFileSync(path.join(here, 'driver.ts'), 'utf8');
const driverRS = fs.readFileSync(path.join(here, 'driver.rs'), 'utf8');
const engines = (id) => [
 {id: 'native', command: [path.join(build, id)]},
 {id: 'node-tsc', command: [process.execPath, path.join(build, `${id}.js`)]},
 {id: 'node-ts', command: [process.execPath, path.join(build, `${id}.ts`)]},
 {id: 'tsx', command: [...tsx, '--tsconfig', path.join(here, 'tsconfig.json'), path.join(build, `${id}.ts`)]},
 {id: 'ts-node-check', command: [...tsnode, '--project', path.join(here, 'tsconfig.json'), path.join(build, `${id}.ts`)]},
 {id: 'ts-node-transpile', command: [...tsnode, '--transpileOnly', '--project', path.join(here, 'tsconfig.json'), path.join(build, `${id}.ts`)]},
 {id: 'bun', command: ['bun', 'run', path.join(build, `${id}.ts`)]},
 {id: 'deno', command: [deno, 'run', '--no-config', '--no-check', '--no-remote', '--no-npm', path.join(build, `${id}.ts`)]}
].filter(engine=>plan.engineIDs.includes(engine.id));
const compilation = [], execution = [], sources = [];
const approximateReferences=new Map(),approximateProbes=new Map(),approximateCorrectness=[];
function prepare(w) {
 const source = fs.readFileSync(path.join(here, 'workloads', `${w.id}.tsr`), 'utf8');
 const ts = fs.readFileSync(path.join(here, 'workloads', `${w.id}.ts`), 'utf8');
 // Refuse emitter changes instead of silently timing the wrong function.
 if(w.family==='testing-assertions'){assertTestingAssertionSource(source,w,'native');assertTestingAssertionSource(ts,w,'typescript');}
 if(w.family==='optional-default-parameters'){assertOptionalDefaultParameterSource(source,w,'native');assertOptionalDefaultParameterSource(ts,w,'typescript');}
 if(w.family==='rest-parameters'){assertRestParameterSource(source,w,'native');assertRestParameterSource(ts,w,'typescript');}
 if(w.family==='array-keys-values'){assertArrayKeysValuesSource(source,w,'native');assertArrayKeysValuesSource(ts,w,'typescript');}
 if(w.family==='string-locale-case'){assertStringLocaleCaseSource(source,w,'native');assertStringLocaleCaseSource(ts,w,'typescript');}
 if(w.family==='string-raw'){assertStringRawSource(source,w,'native');assertStringRawSource(ts,w,'typescript');}
 if(w.family==='number-locale-string'){assertNumberLocaleStringSource(source,w,'native');assertNumberLocaleStringSource(ts,w,'typescript');}
 if(w.family==='array-locale-string'){assertArrayLocaleStringSource(source,w,'native');assertArrayLocaleStringSource(ts,w,'typescript');}
 if(w.family==='destructuring'){assertDestructuringSource(source,w,'native');assertDestructuringSource(ts,w,'typescript');}
 if(w.family==='object-values-entries'){assertObjectValuesSource(source,w,'native');assertObjectValuesSource(ts,w,'typescript');}
 if(w.family==='union-array-elements'){assertUnionArraySource(source,w,'native');assertUnionArraySource(ts,w,'typescript');}
 if(w.family==='data-unions'){assertDataUnionsSource(source,w,'native');assertDataUnionsSource(ts,w,'typescript');}
 if(w.family==='borrowed-optional-fallback'){assertBorrowedOptionalFallbackSource(source,w,'native');assertBorrowedOptionalFallbackSource(ts,w,'typescript');}
 if(w.family==='deferred-initialization'){assertDeferredInitializationSource(source,w,'native');assertDeferredInitializationSource(ts,w,'typescript');}
 if(w.family==='in-operator'){assertInOperatorSource(source,w,'native');assertInOperatorSource(ts,w,'typescript');}
 if(w.family==='callable-expressions'){assertCallableExpressionsSource(source,w,'native');assertCallableExpressionsSource(ts,w,'typescript');}
 if(w.family==='sets'){assertSetsSource(source,w,'native');assertSetsSource(ts,w,'typescript');}
 if(w.family==='maps'){assertMapsSource(source,w,'native');assertMapsSource(ts,w,'typescript');}
 if(w.family==='map-snapshots'){assertMapSnapshotsSource(source,w,'native');assertMapSnapshotsSource(ts,w,'typescript');}
 if(w.family==='set-entries'){assertSetEntriesSource(source,w,'native');assertSetEntriesSource(ts,w,'typescript');}
 if(w.family==='set-for-each'){assertSetForEachSource(source,w,'native');assertSetForEachSource(ts,w,'typescript');}
 if(w.family==='string-iteration'){assertStringIterationSource(source,w,'native');assertStringIterationSource(ts,w,'typescript');}
 if(w.family==='array-from-collections'){assertArrayFromCollectionsSource(source,w,'native');assertArrayFromCollectionsSource(ts,w,'typescript');}
 if(w.family==='iterable-array-spread'){assertIterableArraySpreadSource(source,w,'native');assertIterableArraySpreadSource(ts,w,'typescript');}
 if(w.family==='iterable-call-spread'){assertIterableCallSpreadSource(source,w,'native');assertIterableCallSpreadSource(ts,w,'typescript');}
 if(w.family==='map-group-by'){assertMapGroupBySource(source,w,'native');assertMapGroupBySource(ts,w,'typescript');}
 if(w.family==='comma-contexts'){assertCommaContextsSource(source,w,'native');assertCommaContextsSource(ts,w,'typescript');}
 if(w.family==='comma-expressions'){assertCommaExpressionsSource(source,w,'native');assertCommaExpressionsSource(ts,w,'typescript');}
 if(w.family==='string-indexing'){assertStringIndexingSource(source,w,'native');assertStringIndexingSource(ts,w,'typescript');}
 if(w.family==='optional-fields'){assertOptionalFieldsSource(source,w,'native');assertOptionalFieldsSource(ts,w,'typescript');}
 if(w.family==='interfaces'){assertInterfacesSource(source,w,'native');assertInterfacesSource(ts,w,'typescript');}
 if(w.family==='string-concatenation'){assertStringConcatenationSource(source,w,'native');assertStringConcatenationSource(ts,w,'typescript');}
 if(w.family==='scalar-unary-plus'){assertScalarUnaryPlusSource(source,w,'native');assertScalarUnaryPlusSource(ts,w,'typescript');}
 if(w.family==='nested-function-owned-escape'){assertNestedOwnedSource(source,w,'native');assertNestedOwnedSource(ts,w,'typescript');}
 if(w.family==='nested-function-results'){assertNestedResultSource(source,w,'native');assertNestedResultSource(ts,w,'typescript');}
 if(w.family==='generic-value-adapters'){assertGenericAdapterSource(source,w,'native');assertGenericAdapterSource(ts,w,'typescript');}
 if(w.family==='generic-function-results'){assertGenericResultSource(source,w,'native');assertGenericResultSource(ts,w,'typescript');}
 if(w.family==='generic-function-captures'){assertCapturedGenericSource(source,w,'native');assertCapturedGenericSource(ts,w,'typescript');}
 if(w.family==='generic-function-deferred'){assertDeferredGenericSource(source,w,'native');assertDeferredGenericSource(ts,w,'typescript');}
 if(w.family==='generic-function-contexts'){assertGenericContextSource(source,w,'native');assertGenericContextSource(ts,w,'typescript');}
 if(w.family==='nested-function-rest'){assertNestedRestSource(source,w,'native');assertNestedRestSource(ts,w,'typescript');}
 if(w.family==='nested-function-defaults'){assertNestedDefaultSource(source,w,'native');assertNestedDefaultSource(ts,w,'typescript');}
 if(w.family==='nested-function-captures'){assertNestedFunctionCapturesSource(source,w,'native');assertNestedFunctionCapturesSource(ts,w,'typescript');}
 if(w.family==='nested-function-declarations'){assertNestedFunctionDeclarationsSource(source,w,'native');assertNestedFunctionDeclarationsSource(ts,w,'typescript');}
 if(w.family==='numeric-string-conversion'){assertNumericStringConversionSource(source,w,'native');assertNumericStringConversionSource(ts,w,'typescript');}
 if(w.family==='type-aliases'){assertTypeAliasesSource(source,w,'native');assertTypeAliasesSource(ts,w,'typescript');}
 if(w.family==='numeric-literals'){assertNumericLiteralsSource(source,w,'native');assertNumericLiteralsSource(ts,w,'typescript');}
 if(w.family==='map-iteration'){assertMapIterationSource(source,w,'native');assertMapIterationSource(ts,w,'typescript');}
 if(w.family==='set-algebra'){assertSetAlgebraSource(source,w,'native');assertSetAlgebraSource(ts,w,'typescript');}
 if(w.family==='set-relations'){assertSetRelationsSource(source,w,'native');assertSetRelationsSource(ts,w,'typescript');}
 if(w.family==='map-for-each'){assertMapForEachSource(source,w,'native');assertMapForEachSource(ts,w,'typescript');}
 if(w.family==='array-callback-errors'){assertArrayCallbackErrorsSource(source,w,'native');assertArrayCallbackErrorsSource(ts,w,'typescript');}
 if(w.family==='enums'){assertEnumsSource(source,w,'native');assertEnumsSource(ts,w,'typescript');}
 if(w.family==='type-assertions'){assertTypeAssertionsSource(source,w,'native');assertTypeAssertionsSource(ts,w,'typescript');}
 if(w.family==='exception-completion'){assertExceptionCompletionSource(source,w,'native');assertExceptionCompletionSource(ts,w,'typescript');}
 if(w.family==='labeled-control'){assertLabeledControlSource(source,w,'native');assertLabeledControlSource(ts,w,'typescript');}
 if(w.family==='global-number-names'){assertGlobalNumberNamesSource(source,w,'native');assertGlobalNumberNamesSource(ts,w,'typescript');}
 if(w.family==='function-expressions'){assertFunctionExpressionsSource(source,w,'native');assertFunctionExpressionsSource(ts,w,'typescript');}
 if(w.family==='switch-statements'){assertSwitchStatementsSource(source,w,'native');assertSwitchStatementsSource(ts,w,'typescript');}
 if(w.family==='braceless-bodies'){assertBracelessBodiesSource(source,w,'native');assertBracelessBodiesSource(ts,w,'typescript');}
 if(w.family==='destructuring-assignment'){assertDestructuringAssignmentSource(source,w,'native');assertDestructuringAssignmentSource(ts,w,'typescript');}
 if(w.family==='declaration-lists'){assertDeclarationListsSource(source,w,'native');assertDeclarationListsSource(ts,w,'typescript');}
 if(w.family==='for-pattern-initializers'){assertForPatternInitializersSource(source,w,'native');assertForPatternInitializersSource(ts,w,'typescript');}
 if(w.family==='nested-rest-patterns'){assertNestedRestPatternsSource(source,w,'native');assertNestedRestPatternsSource(ts,w,'typescript');}
 if(w.family==='destructuring-rest'){assertDestructuringRestSource(source,w,'native');assertDestructuringRestSource(ts,w,'typescript');}
 if(w.family==='object-has-own'){assertObjectHasOwnSource(source,w,'native');assertObjectHasOwnSource(ts,w,'typescript');}
 if(w.family==='destructuring-defaults'){assertDestructuringDefaultsSource(source,w,'native');assertDestructuringDefaultsSource(ts,w,'typescript');}
 if(w.family==='object-keys'){assertObjectKeysSource(source,w,'native');assertObjectKeysSource(ts,w,'typescript');}
 if(w.family==='array-entries'){assertArrayEntriesSource(source,w,'native');assertArrayEntriesSource(ts,w,'typescript');}
 if(w.family==='tuple-types'){assertTupleTypesSource(source,w,'native');assertTupleTypesSource(ts,w,'typescript');}
 if(w.family==='string-normalize'){assertStringNormalizeSource(source,w,'native');assertStringNormalizeSource(ts,w,'typescript');}
 if(w.family==='string-code-at'){assertStringCodeAtSource(source,w,'native');assertStringCodeAtSource(ts,w,'typescript');}
 if(w.family==='string-from-codes'){assertStringFromCodesSource(source,w,'native');assertStringFromCodesSource(ts,w,'typescript');}
 if(w.family==='math-random'){assertMathRandomSource(source,w,'native');assertMathRandomSource(ts,w,'typescript');}
 if(w.family==='math-f16round'){assertMathF16roundSource(source,w,'native');assertMathF16roundSource(ts,w,'typescript');}
 if(w.family==='object-is'){assertObjectIsSource(source,w,'native');assertObjectIsSource(ts,w,'typescript');}
 if(w.family==='number-to-string'){assertNumberToStringSource(source,w,'native');assertNumberToStringSource(ts,w,'typescript');}
 if(w.family==='number-parse-int'){assertNumberParseIntSource(source,w,'native');assertNumberParseIntSource(ts,w,'typescript');}
 if(w.family==='string-at'){assertStringAtSource(source,w,'native');assertStringAtSource(ts,w,'typescript');}
 if(w.family==='array-from'){assertArrayFromSource(source,w,'native');assertArrayFromSource(ts,w,'typescript');}
 if(w.family==='array-literal-spread'){assertArrayLiteralSpreadSource(source,w,'native');assertArrayLiteralSpreadSource(ts,w,'typescript');}
 if(w.family==='call-spread'){assertCallSpreadSource(source,w,'native');assertCallSpreadSource(ts,w,'typescript');}
 if(w.id.startsWith('nullable_callable_')){assertNullableCallableSource(source,w,'native');assertNullableCallableSource(ts,w,'typescript');}
 if(/^array_find_(?:last_)?(?:int|number|boolean|string|nested|record|null|undefined)$/.test(w.id)){assertFindValueSource(source,w,'native');assertFindValueSource(ts,w,'typescript');}
 if(/^array_unshift_(?:int|number|boolean|string|nested|record|null|undefined)$/.test(w.id)){assertUnshiftSource(source,w,'native');assertUnshiftSource(ts,w,'typescript');}
 if(/^array_splice_(?:int|number|boolean|string|nested|record|null|undefined)$/.test(w.id)){assertSpliceSource(source,w,'native');assertSpliceSource(ts,w,'typescript');}
 if(/^array_(?:sort|to_sorted)_(?:default|compare)_(?:int|number|boolean|string|nested|record|null|undefined)$/.test(w.id)){assertSortingSource(source,w,'native');assertSortingSource(ts,w,'typescript');}
 if(/^array_(?:flat|flat_map)_(?:int|number|boolean|string|nested|record|null|undefined)$/.test(w.id)){assertFlatSource(source,w,'native');assertFlatSource(ts,w,'typescript');}
 if(/^array_of_(?:int|number|boolean|string|nested|record|null|undefined)$/.test(w.id)){assertOfSource(source,w,'native');assertOfSource(ts,w,'typescript');}
 if(/^nullish_(?:(?:int|number|boolean|string|array|record|null_undefined)_value|nullable_fallback_int)$/.test(w.id)){assertNullishSource(source,w,'native');assertNullishSource(ts,w,'typescript');}
 if(/^nullish_assign_(?:int|number|boolean|string|array|record|nullable_int|nullable_string)_value$/.test(w.id)){assertNullishAssignmentSource(source,w,'native');assertNullishAssignmentSource(ts,w,'typescript');}
 if(/^optional_chain_(?:int_field|number_field|boolean_field|string_length|array_index|array_join|record_field|nullable_chain)$/.test(w.id)){assertOptionalChainSource(source,w,'native');assertOptionalChainSource(ts,w,'typescript');}
 if(w.id.startsWith('array_at_')){assertArrayAtSource(source,w,'native');assertArrayAtSource(ts,w,'typescript');}
 if(/^array_(pop|shift)_/.test(w.id)){assertArrayRemovalSource(source,w,'native');assertArrayRemovalSource(ts,w,'typescript');}
 if(w.id.startsWith('narrowing_')){assertNarrowingSource(source,w,'native');assertNarrowingSource(ts,w,'typescript');}
 if(w.id.startsWith('nullable_')&&!w.id.startsWith('nullable_callable_')){assertNullableTransportSource(source,w,'native');assertNullableTransportSource(ts,w,'typescript');}
 if(w.id.startsWith('array_to_string_')){assertArrayToStringSource(source,w,'native');assertArrayToStringSource(ts,w,'typescript');}
 if(w.id.startsWith('array_join_')){assertArrayJoinSource(source,w,'native');assertArrayJoinSource(ts,w,'typescript');}
 if(['boolean_string_truthiness','string_explicit_int','string_array_conversion'].includes(w.id)){assertValueTypesSource(source,w,'native');assertValueTypesSource(ts,w,'typescript');}
 if(w.id.startsWith('string_html_')){assertHtmlStringSource(source,w,'native');assertHtmlStringSource(ts,w,'typescript');}
 if(w.id.startsWith('string_substr_')){assertStringSubstrSource(source,w,'native');assertStringSubstrSource(ts,w,'typescript');}
 if(['string_cow_split_pieces','string_scalar_split_unicode'].includes(w.id)){assertSplitOwnershipSource(source,w,'native');assertSplitOwnershipSource(ts,w,'typescript');}
 if(w.id.startsWith('template_')){assertTemplateStringSource(source,w,'native');assertTemplateStringSource(ts,w,'typescript');}
 if(w.id.startsWith('quoted_string_literals_')){assertQuotedStringLiteralSource(source,w,'native');assertQuotedStringLiteralSource(ts,w,'typescript');}
 if(/^string_(is|to)_well_formed_/.test(w.id)){assertStringWellFormedSource(source,w,'native');assertStringWellFormedSource(ts,w,'typescript');}
 if(w.id.startsWith('number_parse_float_')){assertNumberParseFloatSource(source,w,'native');assertNumberParseFloatSource(ts,w,'typescript');}
 if(/^number_to_(fixed|exponential|precision)_/.test(w.id)){assertNumberFormatSource(source,w,'native');assertNumberFormatSource(ts,w,'typescript');}
 if(w.id.startsWith('int_to_string_')){assertIntToStringSource(source,w,'native');assertIntToStringSource(ts,w,'typescript');}
 if(/^(string_(to_string|value_of)_(ascii|unicode)|boolean_(to_string|value_of)|number_value_of|int_value_of)$/.test(w.id)){assertPrimitiveIdentitySource(source,w,'native');assertPrimitiveIdentitySource(ts,w,'typescript');}
 if(w.id.startsWith('string_substring_')){assertStringSubstringSource(source,w,'native');assertStringSubstringSource(ts,w,'typescript');}
 if(/^string_(concat_(zero|variadic)_|pad_(start|end)_)/.test(w.id)){assertStringConcatPaddingSource(source,w,'native');assertStringConcatPaddingSource(ts,w,'typescript');}
 if(w.id.startsWith('conditional_')){assertConditionalSource(source,w);assertConditionalSource(ts,w);}
 if(w.id.startsWith('compound_value_')){const expression='const observed: '+w.family+' = (value '+w.compoundOperator+' '+w.rhsIdentifier+');';for(const text of[source,ts])if(!text.includes(expression)||!text.includes('let value: '+w.family+' = ')||!text.includes('output.push(observed); output.push(value);'))throw Error('Compound assignment value source changed');}
 if(/^(int|number|boolean)_assignment_value$/.test(w.id)){const expression='const observed: '+w.family+' = (target = rhsValues[rhsAt]);';for(const text of[source,ts])if(!text.includes(expression)||!text.includes('let target: '+w.family+' = initialValues[')||!text.includes(w.family==='boolean'?'output.push(resultBit); output.push(targetBit);':'output.push(observed); output.push(target);')||(w.family==='boolean'&&(!text.includes('if (observed) { resultBit = 1; }')||!text.includes('if (target) { targetBit = 1; }'))))throw Error('Scalar assignment value source changed');}
 if(w.id.startsWith('boolean_logical_')){const expression='const observed = (target '+w.logicalAssignmentToken+' rightValues[at]);';for(const text of[source,ts])if(!text.includes(expression)||!text.includes('let target = leftValues[at];')||!text.includes('if (observed) { resultBit = 1; }')||!text.includes('if (target) { targetBit = 1; }')||!text.includes('output.push(resultBit); output.push(targetBit);'))throw Error('Logical assignment source changed');}
 if(w.id.startsWith('int_native_')){const expression=w.operatorForm==='unary'?w.operatorToken+'value':w.operatorForm==='binary'?'value '+w.operatorToken+' other':'value '+w.operatorToken+' other;';if(!source.includes(expression)||!ts.includes(expression)||!source.includes(w.operatorForm==='compound'?'output.push(value);':'output.push(observed);')||!ts.includes(w.operatorForm==='compound'?'output.push(value);':'output.push(observed);'))throw Error('Native int operation source changed');}
 if(isCopying(w)){assertCopyingSource(source,w);assertCopyingTSSource(ts,w);}
 else if(isRange(w)){assertRangeSource(source,w);assertRangeTSSource(ts,w);}
 else if(isHigher(w)){assertHigherSource(source,w);assertHigherTSSource(ts,w);}
 else if(isDeclared(w)){assertDeclaredSource(source,w);assertDeclaredTSSource(ts,w);}
 else if(isConcat(w)){assertConcatSource(source,w);assertConcatTSSource(ts,w);}
 else if(isReduction(w))assertReductionSource(source,w);
 else if(w.id.startsWith('array_for_each_'))assertForEachSource(source,w);
 else if (!source.includes(`function workload(n: int, seed: int${w.inputText === undefined ? '' : ', input: string'}${w.scalarCoordinates ? ', coordinates: int' : ''}): ${w.result}`)) throw new Error('Workload ABI changed');
 if(isApproximate(w)){const loaded=loadApproximateReference(w,{tsrSha256:hash(source),tsSha256:hash(ts)});assertApproximateCases(loaded.reference,plan.correctnessCases(w.iterations));approximateReferences.set(w.id,loaded);}
 const emitted = run([compiler], ['emit', path.join(here, 'workloads', `${w.id}.tsr`)]).stdout;
 const reductionABI=assertABI(emitted, w,{source});
 const instrumented = emitted.replace('fn main()', 'fn source_main()') + '\n' + adaptRustDriver(driverRS, w,{emitted,source});
 fs.writeFileSync(path.join(build, `${w.id}.rs`), instrumented);
 fs.writeFileSync(path.join(build, `${w.id}.ts`), ts + '\n' + adaptTSDriver(driverTS, w));
 const nativeCommand = [...rustc, '--edition=2024', '-C', 'opt-level=3', '-C', 'overflow-checks=on', '-A', 'warnings', path.join(build, `${w.id}.rs`), '-o', path.join(build, w.id)];
 fs.writeFileSync(path.join(build, `${w.id}.json`), JSON.stringify({extends: path.join(here, 'tsconfig.json'), files: [path.join(build, `${w.id}.ts`)], compilerOptions: {outDir: build, ...(w.family==='testing-assertions'?{typeRoots:[path.join(here,'node_modules/@types')]}:{})}}));
 const tsCommand = [...tsc, '--project', path.join(build, `${w.id}.json`)];
 // Prime normal filesystem/tool caches. Compiler bootstrap and downloads are excluded.
 run(nativeCommand); run(tsCommand);
 if(isApproximate(w)){
  const loaded=approximateReferences.get(w.id),probe=approximatePointProbeSources(emitted,ts,w,loaded.reference),id=w.id+'_points';
  fs.writeFileSync(path.join(build,id+'.rs'),probe.rs);fs.writeFileSync(path.join(build,id+'.ts'),probe.ts);
  fs.writeFileSync(path.join(build,id+'.json'),JSON.stringify({extends:path.join(here,'tsconfig.json'),files:[path.join(build,id+'.ts')],compilerOptions:{outDir:build}}));
  const nativeProbeCommand=[...rustc,'--edition=2024','-C','opt-level=3','-C','overflow-checks=on','-A','warnings',path.join(build,id+'.rs'),'-o',path.join(build,id)];
  const tsProbeCommand=[...tsc,'--project',path.join(build,id+'.json')];run(nativeProbeCommand);run(tsProbeCommand);
  const probes={sources:{rustSha256:hash(probe.rs),tsSha256:hash(probe.ts),nativeBinarySha256:hash(fs.readFileSync(path.join(build,id))),compiledTSSha256:hash(fs.readFileSync(path.join(build,id+'.js')))},abi:probe.abi,compileCommands:[nativeProbeCommand,tsProbeCommand],engines:{}};
  for(const engine of engines(id)){const actual=JSON.parse(run(engine.command).stdout);const errors=validateApproximatePointProbe(actual,loaded.reference);probes.engines[engine.id]={command:engine.command,actual,errors};}
  approximateProbes.set(w.id,probes);
 }
 const phases = [
  {phase: 'frontend-check', command: [compiler, 'check', path.join(here, 'workloads', `${w.id}.tsr`)]},
  {phase: 'frontend-emit', command: [compiler, 'emit', path.join(here, 'workloads', `${w.id}.tsr`)]},
  {phase: 'rustc-optimized', command: nativeCommand},
  {phase: 'tsc-check-emit', command: tsCommand}
 ];
 if (plan.compilation) for (const phase of phases) {
  const durations = []; for (let i = 0; i < manifest.compileSamples; i++) durations.push(run(phase.command).ms);
  compilation.push({workload: w.id, ...phase, durations});
 }
 sources.push({...(isCopying(w)?{copyingABI:reductionABI}:isRange(w)?{rangeABI:reductionABI}:isHigher(w)?{higherABI:reductionABI}:isDeclared(w)?{declaredABI:reductionABI}:isConcat(w)?{concatABI:reductionABI}:isReduction(w)?{reductionABI}:reductionABI?{workloadABI:reductionABI}:{}),id: w.id, tsrSha256: hash(source), tsSha256: hash(ts), emittedSha256: hash(emitted), instrumentedSha256: hash(instrumented),...(isApproximate(w)?{approximateCompilation:{nativeCommand,tsCommand,instrumentedTSSha256:hash(fs.readFileSync(path.join(build,w.id+'.ts'))),nativeBinarySha256:hash(fs.readFileSync(path.join(build,w.id))),compiledTSSha256:hash(fs.readFileSync(path.join(build,w.id+'.js')))}}:{})});
}
function execute(engine, iterations, seed, warmups, samples, inputText, workload) {
 const result = run(engine.command, [iterations, seed, warmups, samples, ...(inputText === undefined ? [] : [inputText])]);
 const data = JSON.parse(result.stdout);
 if(workload&&isApproximate(workload)){const loaded=approximateReferences.get(workload.id);const accepted=approximateProbes.get(workload.id)?.engines[engine.id];const validation=validateApproximateMeasurement(data,loaded.reference,{iterations,seed,warmups,samples,pointProbeAccepted:!!accepted});return {...data,approximateObservation:validation,processMs:result.ms};}
 if (!Number.isSafeInteger(data.checksum) || data.durations.length !== samples || data.durations.some(t => !Number.isFinite(t) || t < 0)) throw new Error(`Invalid measurement from ${engine.id}`);
 return {...data, processMs: result.ms};
}
const references = new Map(), exactReferences = new Map();
for (const w of workloads) {
 console.log(`Prepare and validate ${w.id}`);
 prepare(w);
 // Boundary shapes and alternate seeds, including full-size workload, on every engine.
 for (const [iterations, seed] of plan.correctnessCases(w.iterations)) {
  const expected = isApproximate(w)?undefined:expectedChecksum(w.id, iterations, seed, w.inputText);
  for (const engine of engines(w.id)) {
   const result = execute(engine, iterations, seed, 0, 1, w.inputText,w);
   if(isApproximate(w)){approximateCorrectness.push({workload:w.id,engine:engine.id,iterations,seed,...result});continue;}
   if (isCopying(w)) validateCopyingObservation(result,expectedExactResult(w.id,iterations,seed,w.inputText),w);
   else if (isRange(w)) validateRangeObservation(result,expectedExactResult(w.id,iterations,seed,w.inputText),w);
   else if (isHigher(w)) validateHigherObservation(result,expectedExactResult(w.id,iterations,seed,w.inputText),w);
   else if (isDeclared(w)) validateDeclaredObservation(result,expectedExactResult(w.id,iterations,seed,w.inputText),w);
   else if (isConcat(w)) validateConcatObservation(result,expectedExactResult(w.id,iterations,seed,w.inputText),w);
   else if (requiresExactResult(w)) validateExactObservation(result,expectedExactResult(w.id,iterations,seed,w.inputText));
   if (expected !== undefined && expected !== result.checksum) throw new Error(`${w.id}/${engine.id}: ${result.checksum}, expected ${expected}, n=${iterations}, seed=${seed}`);
  }
  if (!isApproximate(w)&&iterations === w.iterations) { references.set(w.id, expected); if (requiresExactResult(w)) exactReferences.set(w.id,expectedExactResult(w.id,iterations,seed,w.inputText).exactBits); }
 }
}
const startup = [];
if (!verify) {
 if (plan.startupSamples > 0) {
 // Empty program startup-to-output: distinct from per-batch execution.
 fs.writeFileSync(path.join(build, 'startup.ts'), 'console.log(0);\n');
 fs.writeFileSync(path.join(build, 'startup.rs'), 'fn main() { println!("0"); }\n');
 run([...rustc, '--edition=2024', '-C', 'opt-level=3', path.join(build, 'startup.rs'), '-o', path.join(build, 'startup')]);
 fs.writeFileSync(path.join(build, 'startup.json'), JSON.stringify({extends: path.join(here, 'tsconfig.json'), files: [path.join(build, 'startup.ts')]}));
 run([...tsc, '--project', path.join(build, 'startup.json')]);
 for (const engine of engines('startup')) run(engine.command);
 for (let sample = 0; sample < plan.startupSamples; sample++) for (const engine of rotated(engines('startup'), sample)) {
  const result = run(engine.command); if (result.stdout !== '0') throw new Error('Startup output mismatch');
  startup.push({engine: engine.id, sample, ms: result.ms, command: engine.command});
 }
 }
 for (let processIndex = 0; processIndex < plan.processes; processIndex++) {
  for (const w of rotated(workloads, processIndex)) {
   for (const engine of rotated(engines(w.id), processIndex)) {
    const data = execute(engine, w.iterations, manifest.seed, plan.warmups, plan.samples, w.inputText,w);
    if(!isApproximate(w)){
    if (isCopying(w)) validateCopyingObservation(data,{checksum:references.get(w.id),exactBits:exactReferences.get(w.id)},w);
    else if (isRange(w)) validateRangeObservation(data,{checksum:references.get(w.id),exactBits:exactReferences.get(w.id)},w);
    else if (isHigher(w)) validateHigherObservation(data,{checksum:references.get(w.id),exactBits:exactReferences.get(w.id)},w);
    else if (isDeclared(w)) validateDeclaredObservation(data,{checksum:references.get(w.id),exactBits:exactReferences.get(w.id)},w);
    else if (isConcat(w)) validateConcatObservation(data,{checksum:references.get(w.id),exactBits:exactReferences.get(w.id)},w);
    else if (requiresExactResult(w)) validateExactObservation(data,{checksum:references.get(w.id),exactBits:exactReferences.get(w.id)});
    if (data.checksum !== references.get(w.id)) throw new Error(`${w.id}/${engine.id}: measured checksum mismatch`);
    }
    execution.push({workload: w.id, engine: engine.id, processIndex, command: engine.command, ...data});
   }
   console.log(`Measured ${w.id}, process ${processIndex + 1}/${manifest.processes}`);
  }
 }
}
function rotated(values, offset) { const at = offset % values.length; return [...values.slice(at), ...values.slice(0, at)]; }
const compilerInputs = compilerSourceInputs(path.join(root, 'crates/tsrust-compiler/src'));
for(const loaded of approximateReferences.values())assertApproximateArtifactsUnchanged(loaded);
const result = {
 schemaVersion: 1, measurementScope: plan.measurementScope, measuredEngines: plan.engineIDs, measurementPlanSha256: hash(fs.readFileSync(path.join(here,'measurement-plan.mjs'))), semanticEpoch: manifest.semanticEpoch, timestamp: new Date().toISOString(), verifiedOnly: verify,
 machine: {platform: os.platform(), release: os.release(), architecture: os.arch(), cpu: os.cpus()[0]?.model, logicalCPUs: os.cpus().length, totalMemoryBytes: os.totalmem()},
 versions, nodeComponents: process.versions, manifest, sources, compilerInputs,
 oracleSha256: hash(fs.readFileSync(path.join(here, 'oracle.mjs'))), adapterSha256: hash(fs.readFileSync(path.join(here, 'adapters.mjs'))), harnessSha256: hash(fs.readFileSync(fileURLToPath(import.meta.url))), driverTSSha256: hash(driverTS), driverRSSha256: hash(driverRS), lockSha256: hash(fs.readFileSync(path.join(here, 'package-lock.json'))),
 oracleInputs: ['generic-value-adapters-source-contract.mjs','generic-value-adapters-oracle.mjs','generic-function-results-source-contract.mjs','generic-function-results-oracle.mjs','generic-function-captures-source-contract.mjs','generic-function-captures-oracle.mjs','generic-function-deferred-source-contract.mjs','generic-function-deferred-oracle.mjs','generic-function-contexts-source-contract.mjs','generic-function-contexts-oracle.mjs','nested-function-rest-source-contract.mjs','nested-function-rest-oracle.mjs','nested-function-defaults-source-contract.mjs','nested-function-defaults-oracle.mjs','nested-function-results-source-contract.mjs','nested-function-results-oracle.mjs','nested-function-owned-escape-source-contract.mjs','nested-function-owned-escape-oracle.mjs','nested-function-captures-source-contract.mjs','nested-function-captures-oracle.mjs','nested-function-declarations-source-contract.mjs','nested-function-declarations-oracle.mjs','testing-assertions-oracle.mjs','testing-assertions-source-contract.mjs','array-keys-values-source-contract.mjs','array-keys-values-oracle.mjs','union-array-elements-source-contract.mjs','union-array-elements-oracle.mjs','object-values-entries-source-contract.mjs','object-values-entries-oracle.mjs','destructuring-source-contract.mjs','destructuring-oracle.mjs','object-is-source-contract.mjs','object-is-oracle.mjs','math-f16round-source-contract.mjs','math-f16round-oracle.mjs','math-random-source-contract.mjs','math-random-oracle.mjs','string-from-codes-source-contract.mjs','string-from-codes-oracle.mjs','string-code-at-source-contract.mjs','string-code-at-oracle.mjs','string-locale-case-source-contract.mjs','string-locale-case-oracle.mjs','string-raw-source-contract.mjs','string-raw-oracle.mjs','number-locale-string-source-contract.mjs','number-locale-string-oracle.mjs','array-locale-string-source-contract.mjs','array-locale-string-oracle.mjs','data-unions-source-contract.mjs','function-expressions-oracle.mjs','function-expressions-source-contract.mjs','global-number-names-source-contract.mjs','callable-expressions-source-contract.mjs','callable-expressions-oracle.mjs','array-callback-errors-source-contract.mjs','array-callback-errors-oracle.mjs','map-for-each-source-contract.mjs','map-for-each-oracle.mjs','set-relations-source-contract.mjs','set-relations-oracle.mjs','string-iteration-source-contract.mjs','string-iteration-oracle.mjs','iterable-call-spread-source-contract.mjs','comma-contexts-source-contract.mjs','comma-contexts-oracle.mjs','comma-expressions-oracle.mjs','comma-expressions-source-contract.mjs','numeric-literals-oracle.mjs','numeric-literals-source-contract.mjs','type-aliases-source-contract.mjs','interfaces-source-contract.mjs','numeric-string-conversion-oracle.mjs','numeric-string-conversion-source-contract.mjs','scalar-unary-plus-oracle.mjs', 'scalar-unary-plus-source-contract.mjs', 'string-concatenation-oracle.mjs','string-concatenation-source-contract.mjs','interfaces-oracle.mjs','optional-fields-source-contract.mjs','optional-fields-oracle.mjs','string-indexing-source-contract.mjs','string-indexing-oracle.mjs','type-aliases-oracle.mjs','map-group-by-oracle.mjs','map-group-by-source-contract.mjs','iterable-call-spread-oracle.mjs','iterable-array-spread-source-contract.mjs','iterable-array-spread-oracle.mjs','array-from-collections-source-contract.mjs','array-from-collections-oracle.mjs','map-iteration-source-contract.mjs','map-iteration-oracle.mjs','set-algebra-source-contract.mjs','set-algebra-oracle.mjs','set-for-each-source-contract.mjs','set-for-each-oracle.mjs','set-entries-source-contract.mjs','set-entries-oracle.mjs','map-snapshots-source-contract.mjs','map-snapshots-oracle.mjs','maps-source-contract.mjs','maps-oracle.mjs','sets-source-contract.mjs','sets-oracle.mjs','enums-source-contract.mjs','enums-oracle.mjs','type-assertions-oracle.mjs','exception-completion-oracle.mjs','exception-completion-source-contract.mjs','labeled-control-source-contract.mjs','labeled-control-oracle.mjs','in-operator-source-contract.mjs','in-operator-oracle.mjs','deferred-initialization-source-contract.mjs','deferred-initialization-oracle.mjs','global-number-names-oracle.mjs','switch-statements-oracle.mjs','switch-statements-source-contract.mjs','braceless-bodies-oracle.mjs','braceless-bodies-source-contract.mjs','destructuring-assignment-oracle.mjs','destructuring-assignment-source-contract.mjs','declaration-lists-oracle.mjs','declaration-lists-source-contract.mjs','for-pattern-initializers-oracle.mjs','for-pattern-initializers-source-contract.mjs','nested-rest-patterns-oracle.mjs','nested-rest-patterns-source-contract.mjs','destructuring-rest-oracle.mjs','destructuring-rest-source-contract.mjs','object-has-own-oracle.mjs','object-has-own-source-contract.mjs','destructuring-defaults-oracle.mjs','destructuring-defaults-source-contract.mjs','borrowed-optional-fallback-oracle.mjs','borrowed-optional-fallback-source-contract.mjs','data-unions-oracle.mjs','object-keys-source-contract.mjs','object-keys-oracle.mjs','array-entries-source-contract.mjs','array-entries-oracle.mjs','tuple-types-source-contract.mjs','tuple-types-oracle.mjs','string-normalize-source-contract.mjs','string-normalize-oracle.mjs','number-to-string-source-contract.mjs','number-to-string-oracle.mjs','number-parse-int-source-contract.mjs','number-parse-int-oracle.mjs','string-at-source-contract.mjs','string-at-oracle.mjs','array-from-source-contract.mjs','array-from-oracle.mjs','array-literal-spread-source-contract.mjs','array-literal-spread-oracle.mjs','call-spread-source-contract.mjs','call-spread-oracle.mjs','call-spread-values-oracle.mjs','rest-parameters-source-contract.mjs','rest-parameters-oracle.mjs','rest-parameters-values-oracle.mjs','optional-default-parameters-source-contract.mjs','optional-default-parameters-oracle.mjs','optional-default-parameters-values-oracle.mjs','nullable-callable-source-contract.mjs','nullable-callable-oracle.mjs','nullable-callable-values-oracle.mjs','nullable-callable-higher-order-oracle.mjs','optional-chain-source-contract.mjs','optional-chain-oracle.mjs','nullish-assignment-source-contract.mjs','nullish-assignment-oracle.mjs','nullish-source-contract.mjs','nullish-oracle.mjs','array-of-source-contract.mjs','array-of-oracle.mjs','array-flat-source-contract.mjs','array-flat-oracle.mjs','array-sorting-source-contract.mjs','array-sorting-oracle.mjs','array-splice-source-contract.mjs','array-splice-oracle.mjs','array-unshift-source-contract.mjs','array-unshift-oracle.mjs','array-find-values-source-contract.mjs','array-find-values-oracle.mjs','array-at-source-contract.mjs','array-at-oracle.mjs','array-removal-source-contract.mjs','array-removal-oracle.mjs','narrowing-source-contract.mjs','narrowing-oracle.mjs','nullable-transport-source-contract.mjs','nullable-transport-oracle.mjs','array-to-string-source-contract.mjs','array-to-string-oracle.mjs','array-join-source-contract.mjs','array-join-oracle.mjs','value-types-source-contract.mjs','value-types-oracle.mjs','html-string-source-contract.mjs','html-string-oracle.mjs','string-substr-source-contract.mjs','string-substr-oracle.mjs','split-ownership-source-contract.mjs','split-ownership-oracle.mjs','template-string-source-contract.mjs','template-string-oracle.mjs','quoted-string-literals-source-contract.mjs','quoted-string-literals-oracle.mjs','quoted-string-literals-domains.json','string-well-formed-source-contract.mjs','string-well-formed-oracle.mjs','number-parse-float-source-contract.mjs','number-parse-float-oracle.mjs','number-parse-float-domains.json','number-format-source-contract.mjs','number-format-oracle.mjs','number-format-domains.json','int-to-string-source-contract.mjs','int-to-string-oracle.mjs','primitive-identity-source-contract.mjs','primitive-identity-oracle.mjs','string-substring-source-contract.mjs','string-substring-oracle.mjs','string-concat-padding-source-contract.mjs','string-concat-padding-oracle.mjs','conditional-source-contract.mjs','conditional-oracle.mjs','compound-value-oracle.mjs','assignment-value-oracle.mjs','logical-assignment-oracle.mjs','int-native-oracle.mjs','int-native-domains.json','oracle.mjs','number-oracle.mjs','number-operators-oracle.mjs','number-operators-domains.json','number-predicate-oracle.mjs','math-oracle.mjs','exact-math-oracle.mjs','slice-oracle.mjs','temporary-index-oracle.mjs','map-filter-oracle.mjs','fixed-number-oracle.mjs','array-predicate-oracle.mjs','string-construction-oracle.mjs','reduction-oracle.mjs','reduction-results.mjs','for-each-oracle.mjs','concat-oracle.mjs','declared-value-oracle.mjs','higher-order-oracle.mjs','range-mutation-oracle.mjs','copying-array-oracle.mjs','to-spliced-oracle.mjs','last-index-of-oracle.mjs','string-last-index-of-oracle.mjs','string-trim-oracle.mjs','string-add-oracle.mjs','compound-oracle.mjs','update-oracle.mjs'].map(name=>[name,hash(fs.readFileSync(path.join(here,name)))]), copyingObserverProtocol:'copying-array-natural-v1', copyingAdapterInputs:Object.entries(copyingAdapterSources).map(([name,source])=>[name,hash(source)]), rangeObserverProtocol:'range-array-natural-v1', rangeAdapterInputs:Object.entries(rangeAdapterSources).map(([name,source])=>[name,hash(source)]), higherObserverProtocol:'higher-order-natural-values-v1', higherAdapterInputs:Object.entries(higherAdapterSources).map(([name,source])=>[name,hash(source)]), declaredObserverProtocol:'declared-function-values-v1', declaredAdapterInputs:Object.entries(declaredAdapterSources).map(([name,source])=>[name,hash(source)]), concatObserverProtocol:'declared-concat-v1', concatAdapterInputs:Object.entries(concatAdapterSources).map(([name,source])=>[name,hash(source)]), reductionResultEncoding:'reduction-schema-v1', reductionAdapterInputs:Object.entries(reductionAdapterSources).map(([name,source])=>[name,hash(source)]), arrayResultEncodings: ['array-shape-scalar-v1','ieee-f64-tree-v1'], sliceAdapterInputs: [...Object.entries(sliceAdapterSources),...['slice-observation.mjs','slice-oracle.mjs','result-observation.mjs'].map(name=>[name,fs.readFileSync(path.join(here,name),'utf8')])].map(([name,source])=>[name,hash(source)]), numberResultEncoding: 'ieee-f64-bits-v1', numberAdapterInputs: [...Object.entries(numberAdapterSources), ...['number-observation.mjs','number-oracle.mjs'].map(name=>[name,fs.readFileSync(path.join(here,name),'utf8')])].map(([name,source])=>[name,hash(source)]),
 ...(approximateReferences.size?{approximateInputs:{protocol:'mpfr-directed-f64-scalar-v1',supportHashes:Object.fromEntries(['approximate-harness.mjs','approximate-scalar.mjs',...Object.keys(approximateAdapterSources)].map(name=>[name,hash(fs.readFileSync(path.join(here,name)))])),workloads:Object.fromEntries([...approximateReferences].map(([id,loaded])=>[id,{artifacts:loaded.snapshot,probes:approximateProbes.get(id)}])),correctness:approximateCorrectness}}:{}),
 correctness: {casesPerWorkload: 5, checksums: Object.fromEntries(references), exactBits: Object.fromEntries(exactReferences)}, startup, compilation, execution
};
const file = path.join(here, 'results', `${result.timestamp.replace(/[:.]/g, '-')}${verify ? '-verify' : ''}.json`);
fs.writeFileSync(file, JSON.stringify(result, null, 2) + '\n', {flag: 'wx'});
console.log(`Saved ${file}`);
if (process.argv.includes('--report') && !process.argv.includes('--defer-report')) {
  if (!verify) {
   const {render} = await import('./report.mjs');
   render(file);
  }
  else {
   const {updateReport} = await import('./report.mjs');
   updateReport();
   console.log(`Updated ${path.join(root, 'BENCHMARKS.md')}`);
  }
}
