import {compilerExecutable} from './compiler-executable.mjs';
import {assertGenericReplacementSource} from './generic-callable-replacement-source-contract.mjs';
import {pathRelativeIds,assertPathRelativeSource} from './path-relative-source-contract.mjs';
import {isReduction} from './reduction-observation.mjs';
import {pathJoinIds,assertPathJoinSource} from './path-join-source-contract.mjs';
import {osIdentityIds,assertOsIdentitySource} from './os-identity-source-contract.mjs';
import {lexicalPathIds,assertLexicalPathSource} from './lexical-path-source-contract.mjs';
import {assertGenericAdapterSource} from './generic-value-adapters-source-contract.mjs';
import {assertScopedCallableRecordsSource} from './scoped-callable-records-source-contract.mjs';
import {assertRecursiveResultInferenceSource} from './recursive-result-inference-source-contract.mjs';
import {assertNamedFunctionExpressionsSource} from './named-function-expressions-source-contract.mjs';
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
import {isCopying,assertCopyingPolicy,validateCopyingObservation} from './copying-array-observation.mjs';
import {assertCopyingSource,assertCopyingTSSource} from './copying-array-result-abi.mjs';
import {isRange,assertRangePolicy,validateRangeObservation} from './range-mutation-observation.mjs';
import {assertRangeSource,assertRangeTSSource} from './range-mutation-result-abi.mjs';
import {isHigher} from './higher-order-observation.mjs';
import {assertHigherTSSource} from './higher-order-result-abi.mjs';
import {isDeclared,assertDeclaredPolicy,validateDeclaredObservation} from './declared-value-observation.mjs';
import {isConcat,validateConcatObservation} from './concat-observation.mjs';
import {isApproximate,loadApproximateReference,assertApproximateCases,assertApproximateArtifactsUnchanged,approximatePointProbeSources,approximateAdapterSources} from './approximate-harness.mjs';
import {validateApproximatePointProbe,validateApproximateAllocation} from './approximate-scalar.mjs';
// Allocation diagnosis for the actual runtime workloads; never used for timing comparisons.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import os from 'node:os';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {expectedChecksum} from './oracle.mjs';
import {assertABI, rustChecksum, adaptRustDriver, assertCLIInputs, numberAdapterSources, sliceAdapterSources, reductionAdapterSources, concatAdapterSources, declaredAdapterSources, higherAdapterSources, rangeAdapterSources, copyingAdapterSources} from './adapters.mjs';
const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '../..');
const build = path.join(root, 'target/runtime-comparisons/profiles');
fs.mkdirSync(build, {recursive: true});
const pins = JSON.parse(fs.readFileSync(path.join(here, 'versions.json')));
const manifest = JSON.parse(fs.readFileSync(path.join(here, 'manifest.json')));
const selected = process.argv.find(arg => arg.startsWith('--only='))?.slice(7).split(',');
const workloads = manifest.workloads.filter(workload => !selected || selected.includes(workload.id));
if (!workloads.length || selected?.some(id => !manifest.workloads.some(workload => workload.id === id))) throw new Error('Unknown profile workload');
assertCLIInputs(workloads);for(const w of workloads)isApproximate(w);
const hash = value => crypto.createHash('sha256').update(value).digest('hex');
const run = command => {
 const result = spawnSync(command[0], command.slice(1).map(String), {cwd: root, encoding: 'utf8', timeout: 120000});
 if (result.error || result.status !== 0) throw new Error(result.error ?? result.stderr);
 return result.stdout.trim();
};
run(['cargo', 'build', '--release', '--offline', '--locked', '--quiet']);
const driver = fs.readFileSync(path.join(here, 'allocation-driver.rs'), 'utf8');
const rows = [];
for (const workload of workloads) {
 const sourcePath = path.join(here, 'workloads', `${workload.id}.tsr`);
 const source = fs.readFileSync(sourcePath);
 if(lexicalPathIds.includes(workload.id)){assertLexicalPathSource(source.toString(),workload);assertLexicalPathSource(fs.readFileSync(path.join(here,workload.counterpart),'utf8'),workload);}
 if(osIdentityIds.includes(workload.id)){assertOsIdentitySource(source.toString(),workload);assertOsIdentitySource(fs.readFileSync(path.join(here,workload.counterpart),'utf8'),workload);}
 if(pathJoinIds.includes(workload.id)){assertPathJoinSource(source.toString(),workload);assertPathJoinSource(fs.readFileSync(path.join(here,workload.counterpart),'utf8'),workload);}
 if(pathRelativeIds.includes(workload.id)){assertPathRelativeSource(source.toString(),workload);assertPathRelativeSource(fs.readFileSync(path.join(here,workload.counterpart),'utf8'),workload);}
 if(/^array_find_(?:last_)?(?:int|number|boolean|string|nested|record|null|undefined)$/.test(workload.id)){const findTS=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertFindValueSource(source.toString(),workload,'native');assertFindValueSource(findTS,workload,'typescript');}
 if(/^array_unshift_(?:int|number|boolean|string|nested|record|null|undefined)$/.test(workload.id)){const findTS=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertUnshiftSource(source.toString(),workload,'native');assertUnshiftSource(findTS,workload,'typescript');}
 if(/^array_splice_(?:int|number|boolean|string|nested|record|null|undefined)$/.test(workload.id)){const findTS=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertSpliceSource(source.toString(),workload,'native');assertSpliceSource(findTS,workload,'typescript');}
 if(/^array_(?:sort|to_sorted)_(?:default|compare)_(?:int|number|boolean|string|nested|record|null|undefined)$/.test(workload.id)){const findTS=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertSortingSource(source.toString(),workload,'native');assertSortingSource(findTS,workload,'typescript');}
 if(/^array_(?:flat|flat_map)_(?:int|number|boolean|string|nested|record|null|undefined)$/.test(workload.id)){const findTS=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertFlatSource(source.toString(),workload,'native');assertFlatSource(findTS,workload,'typescript');}
 if(/^array_of_(?:int|number|boolean|string|nested|record|null|undefined)$/.test(workload.id)){const findTS=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertOfSource(source.toString(),workload,'native');assertOfSource(findTS,workload,'typescript');}
 if(/^nullish_(?:(?:int|number|boolean|string|array|record|null_undefined)_value|nullable_fallback_int)$/.test(workload.id)){const findTS=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertNullishSource(source.toString(),workload,'native');assertNullishSource(findTS,workload,'typescript');}
 if(/^nullish_assign_(?:int|number|boolean|string|array|record|nullable_int|nullable_string)_value$/.test(workload.id)){const findTS=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertNullishAssignmentSource(source.toString(),workload,'native');assertNullishAssignmentSource(findTS,workload,'typescript');}
 if(/^optional_chain_(?:int_field|number_field|boolean_field|string_length|array_index|array_join|record_field|nullable_chain)$/.test(workload.id)){const findTS=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertOptionalChainSource(source.toString(),workload,'native');assertOptionalChainSource(findTS,workload,'typescript');}
 if(workload.id.startsWith('array_at_')){const atTS=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertArrayAtSource(source.toString(),workload,'native');assertArrayAtSource(atTS,workload,'typescript');}
 if(/^array_(pop|shift)_/.test(workload.id)){const removalTS=fs.readFileSync(path.join(here,'workloads',workload.id+'.ts'),'utf8');assertArrayRemovalSource(source.toString(),workload,'native');assertArrayRemovalSource(removalTS,workload,'typescript');}
 if(workload.id.startsWith('narrowing_')){const narrowedTS=fs.readFileSync(path.join(here,'workloads',workload.id+'.ts'),'utf8');assertNarrowingSource(source.toString(),workload,'native');assertNarrowingSource(narrowedTS,workload,'typescript');}
 if(workload.id.startsWith('nullable_')&&!workload.id.startsWith('nullable_callable_')){const nullableTS=fs.readFileSync(path.join(here,'workloads',workload.id+'.ts'),'utf8');assertNullableTransportSource(source.toString(),workload,'native');assertNullableTransportSource(nullableTS,workload,'typescript');}
 if(workload.id.startsWith('array_to_string_')){const arrayTS=fs.readFileSync(path.join(here,'workloads',workload.id+'.ts'),'utf8');assertArrayToStringSource(source.toString(),workload,'native');assertArrayToStringSource(arrayTS,workload,'typescript');}
 if(workload.id.startsWith('array_join_')){const joinTS=fs.readFileSync(path.join(here,'workloads',workload.id+'.ts'),'utf8');assertArrayJoinSource(source.toString(),workload,'native');assertArrayJoinSource(joinTS,workload,'typescript');}
 if(['boolean_string_truthiness','string_explicit_int','string_array_conversion'].includes(workload.id)){const valueTS=fs.readFileSync(path.join(here,'workloads',workload.id+'.ts'),'utf8');assertValueTypesSource(source.toString(),workload,'native');assertValueTypesSource(valueTS,workload,'typescript');}
 if(workload.id.startsWith('string_html_')){const htmlTS=fs.readFileSync(path.join(here,'workloads',workload.id+'.ts'),'utf8');assertHtmlStringSource(source.toString(),workload,'native');assertHtmlStringSource(htmlTS,workload,'typescript');}
 if(workload.id.startsWith('string_substr_')){const substrTS=fs.readFileSync(path.join(here,'workloads',workload.id+'.ts'),'utf8');assertStringSubstrSource(source.toString(),workload,'native');assertStringSubstrSource(substrTS,workload,'typescript');}
 if(['string_cow_split_pieces','string_scalar_split_unicode'].includes(workload.id)){const splitTS=fs.readFileSync(path.join(here,'workloads',workload.id+'.ts'),'utf8');assertSplitOwnershipSource(source.toString(),workload,'native');assertSplitOwnershipSource(splitTS,workload,'typescript');}
 if(workload.id.startsWith('template_')){const templateTS=fs.readFileSync(path.join(here,'workloads',workload.id+'.ts'),'utf8');assertTemplateStringSource(source.toString(),workload,'native');assertTemplateStringSource(templateTS,workload,'typescript');}
 if(workload.id.startsWith('quoted_string_literals_')){const quotedTS=fs.readFileSync(path.join(here,'workloads',workload.id+'.ts'),'utf8');assertQuotedStringLiteralSource(source.toString(),workload,'native');assertQuotedStringLiteralSource(quotedTS,workload,'typescript');}
 if(/^string_(is|to)_well_formed_/.test(workload.id)){const wellTS=fs.readFileSync(path.join(here,'workloads',workload.id+'.ts'),'utf8');assertStringWellFormedSource(source.toString(),workload,'native');assertStringWellFormedSource(wellTS,workload,'typescript');}
 if(workload.id.startsWith('number_parse_float_')){const parseTS=fs.readFileSync(path.join(here,'workloads',workload.id+'.ts'),'utf8');assertNumberParseFloatSource(source.toString(),workload,'native');assertNumberParseFloatSource(parseTS,workload,'typescript');}
 if(/^number_to_(fixed|exponential|precision)_/.test(workload.id)){const formatTS=fs.readFileSync(path.join(here,'workloads',workload.id+'.ts'),'utf8');assertNumberFormatSource(source.toString(),workload,'native');assertNumberFormatSource(formatTS,workload,'typescript');}
 if(workload.id.startsWith('int_to_string_')){const formatTS=fs.readFileSync(path.join(here,'workloads',workload.id+'.ts'),'utf8');assertIntToStringSource(source.toString(),workload,'native');assertIntToStringSource(formatTS,workload,'typescript');}
 if(/^(string_(to_string|value_of)_(ascii|unicode)|boolean_(to_string|value_of)|number_value_of|int_value_of)$/.test(workload.id)){const identityTS=fs.readFileSync(path.join(here,'workloads',workload.id+'.ts'),'utf8');assertPrimitiveIdentitySource(source.toString(),workload,'native');assertPrimitiveIdentitySource(identityTS,workload,'typescript');}
 if(workload.id.startsWith('string_substring_')){const substringTS=fs.readFileSync(path.join(here,'workloads',workload.id+'.ts'),'utf8');assertStringSubstringSource(source.toString(),workload,'native');assertStringSubstringSource(substringTS,workload,'typescript');}
 const higherTS=(isCopying(workload)||isRange(workload)||isHigher(workload))?fs.readFileSync(path.join(here,'workloads',workload.id+'.ts'),'utf8'):undefined;
 if(isCopying(workload))assertCopyingTSSource(higherTS,workload);
 else if(isRange(workload))assertRangeTSSource(higherTS,workload);
 else if(isHigher(workload))assertHigherTSSource(higherTS,workload);
 if(workload.family==='optional-default-parameters'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertOptionalDefaultParameterSource(source.toString(),workload,'native');assertOptionalDefaultParameterSource(counterpart,workload,'typescript');}
 if(workload.family==='rest-parameters'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertRestParameterSource(source.toString(),workload,'native');assertRestParameterSource(counterpart,workload,'typescript');}
 if(workload.family==='array-keys-values'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertArrayKeysValuesSource(source.toString(),workload,'native');assertArrayKeysValuesSource(counterpart,workload,'typescript');}
 if(workload.family==='string-locale-case'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertStringLocaleCaseSource(source.toString(),workload,'native');assertStringLocaleCaseSource(counterpart,workload,'typescript');}
 if(workload.family==='string-raw'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertStringRawSource(source.toString(),workload,'native');assertStringRawSource(counterpart,workload,'typescript');}
 if(workload.family==='number-locale-string'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertNumberLocaleStringSource(source.toString(),workload,'native');assertNumberLocaleStringSource(counterpart,workload,'typescript');}
 if(workload.family==='array-locale-string'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertArrayLocaleStringSource(source.toString(),workload,'native');assertArrayLocaleStringSource(counterpart,workload,'typescript');}
 if(workload.family==='destructuring'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertDestructuringSource(source.toString(),workload,'native');assertDestructuringSource(counterpart,workload,'typescript');}
 if(workload.family==='object-values-entries'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertObjectValuesSource(source.toString(),workload,'native');assertObjectValuesSource(counterpart,workload,'typescript');}
 if(workload.family==='union-array-elements'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertUnionArraySource(source.toString(),workload,'native');assertUnionArraySource(counterpart,workload,'typescript');}
 if(workload.family==='data-unions'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertDataUnionsSource(source.toString(),workload,'native');assertDataUnionsSource(counterpart,workload,'typescript');}
 if(workload.family==='borrowed-optional-fallback'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertBorrowedOptionalFallbackSource(source.toString(),workload,'native');assertBorrowedOptionalFallbackSource(counterpart,workload,'typescript');}
 if(workload.family==='deferred-initialization'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertDeferredInitializationSource(source.toString(),workload,'native');assertDeferredInitializationSource(counterpart,workload,'typescript');}
 if(workload.family==='in-operator'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertInOperatorSource(source.toString(),workload,'native');assertInOperatorSource(counterpart,workload,'typescript');}
 if(workload.family==='callable-expressions'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertCallableExpressionsSource(source.toString(),workload,'native');assertCallableExpressionsSource(counterpart,workload,'typescript');}
 if(workload.family==='sets'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertSetsSource(source.toString(),workload,'native');assertSetsSource(counterpart,workload,'typescript');}
 if(workload.family==='maps'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertMapsSource(source.toString(),workload,'native');assertMapsSource(counterpart,workload,'typescript');}
 if(workload.family==='map-snapshots'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertMapSnapshotsSource(source.toString(),workload,'native');assertMapSnapshotsSource(counterpart,workload,'typescript');}
 if(workload.family==='set-entries'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertSetEntriesSource(source.toString(),workload,'native');assertSetEntriesSource(counterpart,workload,'typescript');}
 if(workload.family==='set-for-each'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertSetForEachSource(source.toString(),workload,'native');assertSetForEachSource(counterpart,workload,'typescript');}
 if(workload.family==='string-iteration'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertStringIterationSource(source.toString(),workload,'native');assertStringIterationSource(counterpart,workload,'typescript');}
 if(workload.family==='array-from-collections'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertArrayFromCollectionsSource(source.toString(),workload,'native');assertArrayFromCollectionsSource(counterpart,workload,'typescript');}
 if(workload.family==='iterable-array-spread'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertIterableArraySpreadSource(source.toString(),workload,'native');assertIterableArraySpreadSource(counterpart,workload,'typescript');}
 if(workload.family==='iterable-call-spread'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertIterableCallSpreadSource(source.toString(),workload,'native');assertIterableCallSpreadSource(counterpart,workload,'typescript');}
 if(workload.family==='map-group-by'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertMapGroupBySource(source.toString(),workload,'native');assertMapGroupBySource(counterpart,workload,'typescript');}
 if(workload.family==='comma-contexts'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertCommaContextsSource(source.toString(),workload,'native');assertCommaContextsSource(counterpart,workload,'typescript');}
 if(workload.family==='comma-expressions'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertCommaExpressionsSource(source.toString(),workload,'native');assertCommaExpressionsSource(counterpart,workload,'typescript');}
 if(workload.family==='string-indexing'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertStringIndexingSource(source.toString(),workload,'native');assertStringIndexingSource(counterpart,workload,'typescript');}
 if(workload.family==='optional-fields'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertOptionalFieldsSource(source.toString(),workload,'native');assertOptionalFieldsSource(counterpart,workload,'typescript');}
 if(workload.family==='interfaces'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertInterfacesSource(source.toString(),workload,'native');assertInterfacesSource(counterpart,workload,'typescript');}
 if(workload.family==='string-concatenation'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertStringConcatenationSource(source.toString(),workload,'native');assertStringConcatenationSource(counterpart,workload,'typescript');}
 if(workload.family==='scalar-unary-plus'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertScalarUnaryPlusSource(source.toString(),workload,'native');assertScalarUnaryPlusSource(counterpart,workload,'typescript');}
 if(workload.family==='nested-function-owned-escape'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertNestedOwnedSource(source.toString(),workload,'native');assertNestedOwnedSource(counterpart,workload,'typescript');}
 if(workload.family==='nested-function-results'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertNestedResultSource(source.toString(),workload,'native');assertNestedResultSource(counterpart,workload,'typescript');}
 if(workload.family==='generic-callable-replacement'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertGenericReplacementSource(source.toString(),workload,'native');assertGenericReplacementSource(counterpart,workload,'typescript');}
 if(workload.family==='named-function-expressions'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertNamedFunctionExpressionsSource(source.toString(),workload,'native');assertNamedFunctionExpressionsSource(counterpart,workload,'typescript');}
 if(workload.family==='recursive-result-inference'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertRecursiveResultInferenceSource(source.toString(),workload,'native');assertRecursiveResultInferenceSource(counterpart,workload,'typescript');}
 if(workload.family==='scoped-callable-records'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertScopedCallableRecordsSource(source.toString(),workload,'native');assertScopedCallableRecordsSource(counterpart,workload,'typescript');}
 if(workload.family==='generic-value-adapters'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertGenericAdapterSource(source.toString(),workload,'native');assertGenericAdapterSource(counterpart,workload,'typescript');}
 if(workload.family==='generic-function-results'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertGenericResultSource(source.toString(),workload,'native');assertGenericResultSource(counterpart,workload,'typescript');}
 if(workload.family==='generic-function-captures'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertCapturedGenericSource(source.toString(),workload,'native');assertCapturedGenericSource(counterpart,workload,'typescript');}
 if(workload.family==='generic-function-deferred'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertDeferredGenericSource(source.toString(),workload,'native');assertDeferredGenericSource(counterpart,workload,'typescript');}
 if(workload.family==='generic-function-contexts'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertGenericContextSource(source.toString(),workload,'native');assertGenericContextSource(counterpart,workload,'typescript');}
 if(workload.family==='nested-function-rest'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertNestedRestSource(source.toString(),workload,'native');assertNestedRestSource(counterpart,workload,'typescript');}
 if(workload.family==='nested-function-defaults'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertNestedDefaultSource(source.toString(),workload,'native');assertNestedDefaultSource(counterpart,workload,'typescript');}
 if(workload.family==='nested-function-captures'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertNestedFunctionCapturesSource(source.toString(),workload,'native');assertNestedFunctionCapturesSource(counterpart,workload,'typescript');}
 if(workload.family==='nested-function-declarations'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertNestedFunctionDeclarationsSource(source.toString(),workload,'native');assertNestedFunctionDeclarationsSource(counterpart,workload,'typescript');}
 if(workload.family==='numeric-string-conversion'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertNumericStringConversionSource(source.toString(),workload,'native');assertNumericStringConversionSource(counterpart,workload,'typescript');}
 if(workload.family==='type-aliases'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertTypeAliasesSource(source.toString(),workload,'native');assertTypeAliasesSource(counterpart,workload,'typescript');}
 if(workload.family==='numeric-literals'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertNumericLiteralsSource(source.toString(),workload,'native');assertNumericLiteralsSource(counterpart,workload,'typescript');}
 if(workload.family==='map-iteration'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertMapIterationSource(source.toString(),workload,'native');assertMapIterationSource(counterpart,workload,'typescript');}
 if(workload.family==='set-algebra'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertSetAlgebraSource(source.toString(),workload,'native');assertSetAlgebraSource(counterpart,workload,'typescript');}
 if(workload.family==='set-relations'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertSetRelationsSource(source.toString(),workload,'native');assertSetRelationsSource(counterpart,workload,'typescript');}
 if(workload.family==='map-for-each'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertMapForEachSource(source.toString(),workload,'native');assertMapForEachSource(counterpart,workload,'typescript');}
 if(workload.family==='array-callback-errors'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertArrayCallbackErrorsSource(source.toString(),workload,'native');assertArrayCallbackErrorsSource(counterpart,workload,'typescript');}
 if(workload.family==='enums'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertEnumsSource(source.toString(),workload,'native');assertEnumsSource(counterpart,workload,'typescript');}
 if(workload.family==='type-assertions'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertTypeAssertionsSource(source.toString(),workload,'native');assertTypeAssertionsSource(counterpart,workload,'typescript');}
 if(workload.family==='exception-completion'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertExceptionCompletionSource(source.toString(),workload,'native');assertExceptionCompletionSource(counterpart,workload,'typescript');}
 if(workload.family==='labeled-control'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertLabeledControlSource(source.toString(),workload,'native');assertLabeledControlSource(counterpart,workload,'typescript');}
 if(workload.family==='global-number-names'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertGlobalNumberNamesSource(source.toString(),workload,'native');assertGlobalNumberNamesSource(counterpart,workload,'typescript');}
 if(workload.family==='function-expressions'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertFunctionExpressionsSource(source.toString(),workload,'native');assertFunctionExpressionsSource(counterpart,workload,'typescript');}
 if(workload.family==='switch-statements'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertSwitchStatementsSource(source.toString(),workload,'native');assertSwitchStatementsSource(counterpart,workload,'typescript');}
 if(workload.family==='braceless-bodies'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertBracelessBodiesSource(source.toString(),workload,'native');assertBracelessBodiesSource(counterpart,workload,'typescript');}
 if(workload.family==='destructuring-assignment'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertDestructuringAssignmentSource(source.toString(),workload,'native');assertDestructuringAssignmentSource(counterpart,workload,'typescript');}
 if(workload.family==='declaration-lists'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertDeclarationListsSource(source.toString(),workload,'native');assertDeclarationListsSource(counterpart,workload,'typescript');}
 if(workload.family==='for-pattern-initializers'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertForPatternInitializersSource(source.toString(),workload,'native');assertForPatternInitializersSource(counterpart,workload,'typescript');}
 if(workload.family==='nested-rest-patterns'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertNestedRestPatternsSource(source.toString(),workload,'native');assertNestedRestPatternsSource(counterpart,workload,'typescript');}
 if(workload.family==='destructuring-rest'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertDestructuringRestSource(source.toString(),workload,'native');assertDestructuringRestSource(counterpart,workload,'typescript');}
 if(workload.family==='object-has-own'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertObjectHasOwnSource(source.toString(),workload,'native');assertObjectHasOwnSource(counterpart,workload,'typescript');}
 if(workload.family==='destructuring-defaults'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertDestructuringDefaultsSource(source.toString(),workload,'native');assertDestructuringDefaultsSource(counterpart,workload,'typescript');}
 if(workload.family==='object-keys'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertObjectKeysSource(source.toString(),workload,'native');assertObjectKeysSource(counterpart,workload,'typescript');}
 if(workload.family==='array-entries'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertArrayEntriesSource(source.toString(),workload,'native');assertArrayEntriesSource(counterpart,workload,'typescript');}
 if(workload.family==='tuple-types'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertTupleTypesSource(source.toString(),workload,'native');assertTupleTypesSource(counterpart,workload,'typescript');}
 if(workload.family==='string-normalize'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertStringNormalizeSource(source.toString(),workload,'native');assertStringNormalizeSource(counterpart,workload,'typescript');}
 if(workload.family==='string-code-at'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertStringCodeAtSource(source.toString(),workload,'native');assertStringCodeAtSource(counterpart,workload,'typescript');}
 if(workload.family==='string-from-codes'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertStringFromCodesSource(source.toString(),workload,'native');assertStringFromCodesSource(counterpart,workload,'typescript');}
 if(workload.family==='math-random'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertMathRandomSource(source.toString(),workload,'native');assertMathRandomSource(counterpart,workload,'typescript');}
 if(workload.family==='math-f16round'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertMathF16roundSource(source.toString(),workload,'native');assertMathF16roundSource(counterpart,workload,'typescript');}
 if(workload.family==='object-is'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertObjectIsSource(source.toString(),workload,'native');assertObjectIsSource(counterpart,workload,'typescript');}
 if(workload.family==='number-to-string'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertNumberToStringSource(source.toString(),workload,'native');assertNumberToStringSource(counterpart,workload,'typescript');}
 if(workload.family==='number-parse-int'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertNumberParseIntSource(source.toString(),workload,'native');assertNumberParseIntSource(counterpart,workload,'typescript');}
 if(workload.family==='string-at'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertStringAtSource(source.toString(),workload,'native');assertStringAtSource(counterpart,workload,'typescript');}
 if(workload.family==='array-from'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertArrayFromSource(source.toString(),workload,'native');assertArrayFromSource(counterpart,workload,'typescript');}
 if(workload.family==='array-literal-spread'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertArrayLiteralSpreadSource(source.toString(),workload,'native');assertArrayLiteralSpreadSource(counterpart,workload,'typescript');}
 if(workload.family==='call-spread'){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertCallSpreadSource(source.toString(),workload,'native');assertCallSpreadSource(counterpart,workload,'typescript');}
 if(workload.id.startsWith('nullable_callable_')){const counterpart=fs.readFileSync(path.join(here,workload.counterpart),'utf8');assertNullableCallableSource(source.toString(),workload,'native');assertNullableCallableSource(counterpart,workload,'typescript');}
 const emitted = run([compilerExecutable(root, root), 'emit', sourcePath]);
 const reductionABI=assertABI(emitted, workload,{source:source.toString()});
 if(isApproximate(workload)){
  const ts=fs.readFileSync(path.join(here,'workloads',workload.id+'.ts'),'utf8');
  const loaded=loadApproximateReference(workload,{tsSha256:hash(ts),tsrSha256:hash(source)});assertApproximateCases(loaded.reference,[[workload.iterations,7]]);
  const probe=approximatePointProbeSources(emitted,ts,workload,loaded.reference),probeSource=path.join(build,workload.id+'_points.rs'),probeBinary=path.join(build,workload.id+'_points');
  fs.writeFileSync(probeSource,probe.rs);const probeCommand=['rustup','run',pins.rust,'rustc','--edition=2024','-C','opt-level=3','-C','overflow-checks=on','-A','warnings',probeSource,'-o',probeBinary];run(probeCommand);
  const actualProbe=JSON.parse(run([probeBinary])),pointErrors=validateApproximatePointProbe(actualProbe,loaded.reference);
  const instrumented=emitted.replace('fn main()','fn source_main()')+'\n'+approximateAdapterSources['approximate-allocation-driver.rs'];
  const nativeSource=path.join(build,workload.id+'.rs'),executable=path.join(build,workload.id);fs.writeFileSync(nativeSource,instrumented);
  const command=['rustup','run',pins.rust,'rustc','--edition=2024','-C','opt-level=3','-C','overflow-checks=on','-A','warnings',nativeSource,'-o',executable];run(command);
  const samples=Array.from({length:3},()=>{const actual=JSON.parse(run([executable,workload.iterations,7]));return {...actual,approximateObservation:validateApproximateAllocation(actual,loaded.reference,{iterations:workload.iterations,seed:7,acceptedNativePointProbe:pointErrors})};});
  assertApproximateArtifactsUnchanged(loaded);rows.push({workload:workload.id,iterations:workload.iterations,seed:7,resultEncoding:workload.resultEncoding,sourceSha256:hash(source),tsSha256:hash(ts),emittedSha256:hash(emitted),instrumentedSha256:hash(instrumented),binarySha256:hash(fs.readFileSync(executable)),commands:{profile:command,probe:probeCommand},approximateInputs:{artifacts:loaded.snapshot,probe:{actual:actualProbe,errors:pointErrors,abi:probe.abi,sourceSha256:hash(probe.rs),binarySha256:hash(fs.readFileSync(probeBinary))},supportHashes:Object.fromEntries(['approximate-harness.mjs','approximate-scalar.mjs','approximate-allocation-driver.rs'].map(name=>[name,hash(fs.readFileSync(path.join(here,name)))]))},samples});
  console.log('Profiled '+workload.id+': '+samples[0].allocations+' allocations, '+samples[0].reallocations+' reallocations');continue;
 }
 const instrumented = emitted.replace('fn main()', 'fn source_main()') + '\n' + rustChecksum(workload, {probe: true,emitted,source:source.toString()}) + '\n' + adaptRustDriver(driver, workload, {probe: true,emitted,source:source.toString()});
 const nativeSource = path.join(build, `${workload.id}.rs`);
 const executable = path.join(build, workload.id);
 fs.writeFileSync(nativeSource, instrumented);
 run(['rustup', 'run', pins.rust, 'rustc', '--edition=2024', '-C', 'opt-level=3', '-C', 'overflow-checks=on', '-A', 'warnings', nativeSource, '-o', executable]);
 const samples = Array.from({length: 3}, () => JSON.parse(run([executable, workload.iterations, ...(workload.inputText === undefined ? [] : [workload.inputText])])));
 for (const sample of samples) if (sample.checksum !== expectedChecksum(workload.id, workload.iterations, 7, workload.inputText) || sample.liveBytesAfterDrop !== 0) throw new Error(`Invalid profile: ${workload.id}`);
 rows.push({...(isCopying(workload)?{copyingABI:reductionABI,tsSha256:hash(higherTS)}:isRange(workload)?{rangeABI:reductionABI,tsSha256:hash(higherTS)}:isHigher(workload)?{higherABI:reductionABI,tsSha256:hash(higherTS)}:isDeclared(workload)?{declaredABI:reductionABI}:isConcat(workload)?{concatABI:reductionABI}:isReduction(workload)?{reductionABI}:reductionABI?{workloadABI:reductionABI}:{}),workload: workload.id, iterations: workload.iterations, seed: 7, sourceSha256: hash(source), emittedSha256: hash(emitted), instrumentedSha256: hash(instrumented), samples});
 console.log(`Profiled ${workload.id}: ${samples[0].allocations} allocations, ${samples[0].reallocations} reallocations`);
}
const data = {semanticEpoch: manifest.semanticEpoch, kind: 'native-workload-allocation-diagnosis', machine: {platform: os.platform(), release: os.release(), architecture: os.arch(), cpu: os.cpus()[0]?.model}, timestamp: new Date().toISOString(), rustVersion: run(['rustup','run', pins.rust,'rustc','--version']), driverSha256: hash(driver), oracleInputs: ['recursive-result-inference-source-contract.mjs','recursive-result-inference-oracle.mjs','named-function-expressions-source-contract.mjs','named-function-expressions-oracle.mjs','path-relative-source-contract.mjs','path-join-source-contract.mjs','os-identity-source-contract.mjs','lexical-path-source-contract.mjs','generic-callable-replacement-source-contract.mjs','generic-callable-replacement-oracle.mjs','scoped-callable-records-source-contract.mjs','scoped-callable-records-oracle.mjs','generic-value-adapters-source-contract.mjs','format-source-identity.mjs','format-source-identities.json','source-contract-tokens.mjs','generic-value-adapters-oracle.mjs','generic-function-results-source-contract.mjs','generic-function-results-oracle.mjs','generic-function-captures-source-contract.mjs','generic-function-captures-oracle.mjs','generic-function-deferred-source-contract.mjs','generic-function-deferred-oracle.mjs','generic-function-contexts-source-contract.mjs','generic-function-contexts-oracle.mjs','nested-function-rest-source-contract.mjs','nested-function-rest-oracle.mjs','nested-function-defaults-source-contract.mjs','nested-function-defaults-oracle.mjs','nested-function-results-source-contract.mjs','nested-function-results-oracle.mjs','nested-function-owned-escape-source-contract.mjs','nested-function-owned-escape-oracle.mjs','nested-function-captures-source-contract.mjs','nested-function-captures-oracle.mjs','nested-function-declarations-source-contract.mjs','nested-function-declarations-oracle.mjs','array-literal-spread-source-contract.mjs','array-keys-values-source-contract.mjs','array-keys-values-oracle.mjs','union-array-elements-source-contract.mjs','union-array-elements-oracle.mjs','object-values-entries-source-contract.mjs','object-values-entries-oracle.mjs','destructuring-source-contract.mjs','destructuring-oracle.mjs','object-is-source-contract.mjs','object-is-oracle.mjs','math-f16round-source-contract.mjs','math-f16round-oracle.mjs','math-random-source-contract.mjs','math-random-oracle.mjs','string-from-codes-source-contract.mjs','string-from-codes-oracle.mjs','string-code-at-source-contract.mjs','string-code-at-oracle.mjs','string-locale-case-source-contract.mjs','string-locale-case-oracle.mjs','string-raw-source-contract.mjs','string-raw-oracle.mjs','number-locale-string-source-contract.mjs','number-locale-string-oracle.mjs','array-locale-string-source-contract.mjs','array-locale-string-oracle.mjs','data-unions-source-contract.mjs','function-expressions-oracle.mjs','function-expressions-source-contract.mjs','global-number-names-source-contract.mjs','callable-expressions-source-contract.mjs','callable-expressions-oracle.mjs','array-callback-errors-source-contract.mjs','array-callback-errors-oracle.mjs','map-for-each-source-contract.mjs','map-for-each-oracle.mjs','set-relations-source-contract.mjs','set-relations-oracle.mjs','string-iteration-source-contract.mjs','string-iteration-oracle.mjs','iterable-call-spread-source-contract.mjs','comma-contexts-source-contract.mjs','comma-contexts-oracle.mjs','comma-expressions-oracle.mjs','comma-expressions-source-contract.mjs','numeric-literals-oracle.mjs','numeric-literals-source-contract.mjs','type-aliases-source-contract.mjs','interfaces-source-contract.mjs','numeric-string-conversion-oracle.mjs','numeric-string-conversion-source-contract.mjs','scalar-unary-plus-oracle.mjs', 'scalar-unary-plus-source-contract.mjs', 'string-concatenation-oracle.mjs','string-concatenation-source-contract.mjs','interfaces-oracle.mjs','optional-fields-source-contract.mjs','optional-fields-oracle.mjs','string-indexing-source-contract.mjs','string-indexing-oracle.mjs','type-aliases-oracle.mjs','map-group-by-oracle.mjs','map-group-by-source-contract.mjs','iterable-call-spread-oracle.mjs','iterable-array-spread-source-contract.mjs','iterable-array-spread-oracle.mjs','array-from-collections-source-contract.mjs','array-from-collections-oracle.mjs','map-iteration-source-contract.mjs','map-iteration-oracle.mjs','set-algebra-source-contract.mjs','set-algebra-oracle.mjs','set-for-each-source-contract.mjs','set-for-each-oracle.mjs','set-entries-source-contract.mjs','set-entries-oracle.mjs','map-snapshots-source-contract.mjs','map-snapshots-oracle.mjs','maps-source-contract.mjs','maps-oracle.mjs','sets-source-contract.mjs','sets-oracle.mjs','enums-source-contract.mjs','enums-oracle.mjs','type-assertions-oracle.mjs','exception-completion-oracle.mjs','exception-completion-source-contract.mjs','labeled-control-source-contract.mjs','labeled-control-oracle.mjs','in-operator-source-contract.mjs','in-operator-oracle.mjs','deferred-initialization-source-contract.mjs','deferred-initialization-oracle.mjs','global-number-names-oracle.mjs','switch-statements-oracle.mjs','switch-statements-source-contract.mjs','braceless-bodies-oracle.mjs','braceless-bodies-source-contract.mjs','destructuring-assignment-oracle.mjs','destructuring-assignment-source-contract.mjs','declaration-lists-oracle.mjs','declaration-lists-source-contract.mjs','for-pattern-initializers-oracle.mjs','for-pattern-initializers-source-contract.mjs','nested-rest-patterns-oracle.mjs','nested-rest-patterns-source-contract.mjs','destructuring-rest-oracle.mjs','destructuring-rest-source-contract.mjs','object-has-own-oracle.mjs','object-has-own-source-contract.mjs','destructuring-defaults-oracle.mjs','destructuring-defaults-source-contract.mjs','borrowed-optional-fallback-oracle.mjs','borrowed-optional-fallback-source-contract.mjs','data-unions-oracle.mjs','object-keys-source-contract.mjs','object-keys-oracle.mjs','array-entries-source-contract.mjs','array-entries-oracle.mjs','tuple-types-source-contract.mjs','tuple-types-oracle.mjs','string-normalize-source-contract.mjs','string-normalize-oracle.mjs','number-to-string-source-contract.mjs','number-to-string-oracle.mjs','number-parse-int-source-contract.mjs','number-parse-int-oracle.mjs','string-at-source-contract.mjs','string-at-oracle.mjs','array-from-source-contract.mjs','array-from-oracle.mjs','array-literal-spread-oracle.mjs','call-spread-source-contract.mjs','call-spread-oracle.mjs','call-spread-values-oracle.mjs','rest-parameters-source-contract.mjs','rest-parameters-oracle.mjs','rest-parameters-values-oracle.mjs','optional-default-parameters-source-contract.mjs','optional-default-parameters-oracle.mjs','optional-default-parameters-values-oracle.mjs','nullable-callable-source-contract.mjs','nullable-callable-oracle.mjs','nullable-callable-values-oracle.mjs','nullable-callable-higher-order-oracle.mjs','optional-chain-source-contract.mjs','optional-chain-oracle.mjs','nullish-assignment-source-contract.mjs','nullish-assignment-oracle.mjs','nullish-source-contract.mjs','nullish-oracle.mjs','array-of-source-contract.mjs','array-of-oracle.mjs','array-flat-source-contract.mjs','array-flat-oracle.mjs','array-sorting-source-contract.mjs','array-sorting-oracle.mjs','array-splice-source-contract.mjs','array-splice-oracle.mjs','array-unshift-source-contract.mjs','array-unshift-oracle.mjs','array-find-values-source-contract.mjs','array-find-values-oracle.mjs','array-at-source-contract.mjs','array-at-oracle.mjs','array-removal-source-contract.mjs','array-removal-oracle.mjs','narrowing-source-contract.mjs','narrowing-oracle.mjs','nullable-transport-source-contract.mjs','nullable-transport-oracle.mjs','array-to-string-source-contract.mjs','array-to-string-oracle.mjs','array-join-source-contract.mjs','array-join-oracle.mjs','value-types-source-contract.mjs','value-types-oracle.mjs','html-string-source-contract.mjs','html-string-oracle.mjs','string-substr-source-contract.mjs','string-substr-oracle.mjs','split-ownership-source-contract.mjs','split-ownership-oracle.mjs','template-string-source-contract.mjs','template-string-oracle.mjs','quoted-string-literals-source-contract.mjs','quoted-string-literals-oracle.mjs','quoted-string-literals-domains.json','string-well-formed-source-contract.mjs','string-well-formed-oracle.mjs','number-parse-float-source-contract.mjs','number-parse-float-oracle.mjs','number-parse-float-domains.json','number-format-source-contract.mjs','number-format-oracle.mjs','number-format-domains.json','int-to-string-source-contract.mjs','int-to-string-oracle.mjs','primitive-identity-source-contract.mjs','primitive-identity-oracle.mjs','string-substring-source-contract.mjs','string-substring-oracle.mjs','string-concat-padding-source-contract.mjs','string-concat-padding-oracle.mjs','conditional-source-contract.mjs','conditional-oracle.mjs','compound-value-oracle.mjs','assignment-value-oracle.mjs','logical-assignment-oracle.mjs','int-native-oracle.mjs','int-native-domains.json','oracle.mjs','number-oracle.mjs','number-operators-oracle.mjs','number-operators-domains.json','number-predicate-oracle.mjs','math-oracle.mjs','exact-math-oracle.mjs','slice-oracle.mjs','temporary-index-oracle.mjs','map-filter-oracle.mjs','fixed-number-oracle.mjs','array-predicate-oracle.mjs','string-construction-oracle.mjs','reduction-oracle.mjs','reduction-results.mjs','for-each-oracle.mjs','concat-oracle.mjs','declared-value-oracle.mjs','higher-order-oracle.mjs','range-mutation-oracle.mjs','copying-array-oracle.mjs','to-spliced-oracle.mjs','last-index-of-oracle.mjs','string-last-index-of-oracle.mjs','string-trim-oracle.mjs','string-add-oracle.mjs','compound-oracle.mjs','update-oracle.mjs'].map(name=>[name,hash(fs.readFileSync(path.join(here,name)))]), copyingObserverProtocol:'copying-array-natural-v1', copyingAdapterInputs:Object.entries(copyingAdapterSources).map(([name,source])=>[name,hash(source)]), rangeObserverProtocol:'range-array-natural-v1', rangeAdapterInputs:Object.entries(rangeAdapterSources).map(([name,source])=>[name,hash(source)]), higherObserverProtocol:'higher-order-natural-values-v1', higherAdapterInputs:Object.entries(higherAdapterSources).map(([name,source])=>[name,hash(source)]), declaredObserverProtocol:'declared-function-values-v1', declaredAdapterInputs:Object.entries(declaredAdapterSources).map(([name,source])=>[name,hash(source)]), concatObserverProtocol:'declared-concat-v1', concatAdapterInputs:Object.entries(concatAdapterSources).map(([name,source])=>[name,hash(source)]), reductionResultEncoding:'reduction-schema-v1', reductionAdapterInputs:Object.entries(reductionAdapterSources).map(([name,source])=>[name,hash(source)]), arrayResultEncodings: ['array-shape-scalar-v1','ieee-f64-tree-v1'], sliceAdapterInputs: [...Object.entries(sliceAdapterSources),...['slice-observation.mjs','slice-oracle.mjs','result-observation.mjs'].map(name=>[name,fs.readFileSync(path.join(here,name),'utf8')])].map(([name,source])=>[name,hash(source)]), numberResultEncoding: 'ieee-f64-bits-v1', numberAdapterInputs: [...Object.entries(numberAdapterSources),...['number-observation.mjs','number-oracle.mjs'].map(name=>[name,fs.readFileSync(path.join(here,name),'utf8')])].map(([name,source])=>[name,hash(source)]), adapterSha256: hash(fs.readFileSync(path.join(here, 'adapters.mjs'))), oracleSha256: hash(fs.readFileSync(path.join(here, 'oracle.mjs'))), manifest, harnessSha256: hash(fs.readFileSync(fileURLToPath(import.meta.url))), compilerInputs: fs.readdirSync(path.join(root,'crates/tsrust-compiler/src')).filter(n => n.endsWith('.rs')).sort().map(n => [n, hash(fs.readFileSync(path.join(root,'crates/tsrust-compiler/src',n)))]), rows};
const destination = path.join(here, 'diagnostics');
fs.mkdirSync(destination, {recursive: true});
const file = path.join(destination, `${data.timestamp.replace(/[:.]/g,'-')}-native-allocations.json`);
fs.writeFileSync(file, JSON.stringify(data,null,2)+'\n', {flag: 'wx'});
console.log(`Saved ${file}`);
