import {expectedNumericStringConversionObservation} from './numeric-string-conversion-oracle.mjs';
import {expectedScalarUnaryPlusObservation} from './scalar-unary-plus-oracle.mjs';
import {expectedStringConcatenationObservation} from './string-concatenation-oracle.mjs';
import {expectedCommaContextsObservation} from './comma-contexts-oracle.mjs';
import {expectedCommaExpressionsObservation} from './comma-expressions-oracle.mjs';
import {expectedStringIndexingObservation} from './string-indexing-oracle.mjs';
import {expectedOptionalFieldsObservation} from './optional-fields-oracle.mjs';
import {expectedInterfacesObservation} from './interfaces-oracle.mjs';
import {expectedTypeAliasesObservation} from './type-aliases-oracle.mjs';
import {expectedNumericLiteralsObservation} from './numeric-literals-oracle.mjs';
import {expectedMapGroupByObservation} from './map-group-by-oracle.mjs';
import {expectedIterableCallSpreadObservation} from './iterable-call-spread-oracle.mjs';
import {expectedIterableArraySpreadObservation} from './iterable-array-spread-oracle.mjs';
import {expectedArrayFromCollectionsObservation} from './array-from-collections-oracle.mjs';
import {expectedStringIterationObservation} from './string-iteration-oracle.mjs';
import {expectedMapIterationObservation} from './map-iteration-oracle.mjs';
import {expectedSetAlgebraObservation} from './set-algebra-oracle.mjs';
import {expectedSetRelationsObservation} from './set-relations-oracle.mjs';
import {expectedMapForEachObservation} from './map-for-each-oracle.mjs';
import {expectedArrayCallbackErrorsObservation} from './array-callback-errors-oracle.mjs';
import {expectedSetForEachObservation} from './set-for-each-oracle.mjs';
import {expectedSetEntriesObservation} from './set-entries-oracle.mjs';
import {expectedMapSnapshotsObservation} from './map-snapshots-oracle.mjs';
import {expectedMapsObservation} from './maps-oracle.mjs';
import {expectedSetsObservation} from './sets-oracle.mjs';
import {expectedEnumsObservation} from './enums-oracle.mjs';
import {expectedTypeAssertionsObservation} from './type-assertions-oracle.mjs';
import {expectedExceptionCompletionObservation} from './exception-completion-oracle.mjs';
import {callableExpressionsIds,expectedCallableExpressionsObservation} from './callable-expressions-oracle.mjs';
import {labeledControlIds,expectedLabeledControlObservation} from './labeled-control-oracle.mjs';
import {inOperatorIds,expectedInOperatorObservation} from './in-operator-oracle.mjs';
import {deferredInitializationIds,expectedDeferredInitializationObservation} from './deferred-initialization-oracle.mjs';
import {globalNumberNamesIds,expectedGlobalNumberNamesObservation} from './global-number-names-oracle.mjs';
import {functionExpressionsIds,expectedFunctionExpressionsObservation} from './function-expressions-oracle.mjs';
import {switchStatementsIds,expectedSwitchStatementsObservation} from './switch-statements-oracle.mjs';
import {bracelessBodiesIds,expectedBracelessBodiesObservation} from './braceless-bodies-oracle.mjs';
import {destructuringAssignmentIds,expectedDestructuringAssignmentObservation} from './destructuring-assignment-oracle.mjs';
import {declarationListsIds,expectedDeclarationListsObservation} from './declaration-lists-oracle.mjs';
import {forPatternInitializersIds,expectedForPatternInitializersObservation} from './for-pattern-initializers-oracle.mjs';
import {nestedRestPatternsIds,expectedNestedRestPatternsObservation} from './nested-rest-patterns-oracle.mjs';
import {destructuringRestIds,expectedDestructuringRestObservation} from './destructuring-rest-oracle.mjs';
import {objectHasOwnIds,expectedObjectHasOwnObservation} from './object-has-own-oracle.mjs';
import {expectedTestingAssertionChecksum,expectedTestingExpectationChecksum} from './testing-assertions-oracle.mjs';
import {destructuringDefaultsIds,expectedDestructuringDefaultsObservation} from './destructuring-defaults-oracle.mjs';
import {borrowedOptionalFallbackIds,expectedBorrowedOptionalFallbackValues,expectedBorrowedOptionalFallbackObservation} from './borrowed-optional-fallback-oracle.mjs';
import {dataUnionsIds,expectedDataUnionsObservation} from './data-unions-oracle.mjs';
import {unionArrayIds,expectedUnionArrayObservation} from './union-array-elements-oracle.mjs';
import {objectValuesIds,expectedObjectValuesObservation} from './object-values-entries-oracle.mjs';
import {destructuringIds,expectedDestructuringObservation} from './destructuring-oracle.mjs';
import {objectKeysIds,expectedObjectKeysObservation} from './object-keys-oracle.mjs';
import {arrayEntriesIds,expectedArrayEntriesObservation} from './array-entries-oracle.mjs';
import {tupleTypesIds,expectedTupleTypesObservation} from './tuple-types-oracle.mjs';
import {stringNormalizeIds,expectedStringNormalizeObservation} from './string-normalize-oracle.mjs';
import {arrayLocaleStringIds,expectedArrayLocaleStringObservation} from './array-locale-string-oracle.mjs';
import {numberLocaleStringIds,expectedNumberLocaleStringObservation} from './number-locale-string-oracle.mjs';
import {stringRawIds,expectedStringRawObservation} from './string-raw-oracle.mjs';
import {arrayFromIds,expectedArrayFromObservation} from './array-from-oracle.mjs';
import {stringAtIds,expectedStringAtObservation} from './string-at-oracle.mjs';
import {numberParseIntIds,expectedNumberParseIntObservation} from './number-parse-int-oracle.mjs';
import {numberToStringIds,expectedNumberToStringObservation} from './number-to-string-oracle.mjs';
import {objectIsIds,expectedObjectIsObservation} from './object-is-oracle.mjs';
import {mathF16roundIds,expectedMathF16roundObservation} from './math-f16round-oracle.mjs';
import {mathRandomIds,expectedMathRandomObservation} from './math-random-oracle.mjs';
import {stringFromCodesIds,expectedStringFromCodesObservation} from './string-from-codes-oracle.mjs';
import {stringCodeAtIds,expectedStringCodeAtObservation} from './string-code-at-oracle.mjs';
import {stringLocaleCaseIds,expectedStringLocaleCaseObservation} from './string-locale-case-oracle.mjs';
import {arrayKeysValuesIds,expectedArrayKeysValuesObservation} from './array-keys-values-oracle.mjs';
import {arrayLiteralSpreadIds,expectedArrayLiteralSpreadObservation} from './array-literal-spread-oracle.mjs';
import {callSpreadIds,expectedCallSpreadObservation} from './call-spread-oracle.mjs';
import {expectedLogicalValueObservation} from './logical-values-oracle.mjs';
import {restParameterIds,expectedRestParameterObservation} from './rest-parameters-oracle.mjs';
import {optionalDefaultParameterIds,expectedOptionalDefaultParameterObservation} from './optional-default-parameters-oracle.mjs';
import {nullableCallableIds,expectedNullableCallableObservation} from './nullable-callable-oracle.mjs';
import {callableFoundationIds, expectedCallableFoundation} from './callable-foundation-oracle.mjs';
import {expectedOptionalChainObservation} from './optional-chain-oracle.mjs';
import {expectedNullishAssignmentObservation} from './nullish-assignment-oracle.mjs';
import {expectedNullishObservation} from './nullish-oracle.mjs';
import {expectedOfObservation} from './array-of-oracle.mjs';
import {expectedFlatObservation} from './array-flat-oracle.mjs';
import {expectedSortingObservation} from './array-sorting-oracle.mjs';
import {expectedSpliceObservation} from './array-splice-oracle.mjs';
import {expectedUnshiftObservation} from './array-unshift-oracle.mjs';
import {expectedFindObservation} from './array-find-values-oracle.mjs';
import {arrayAtIds,expectedArrayAtObservation} from './array-at-oracle.mjs';
import {arrayRemovalIds,expectedArrayRemovalObservation} from './array-removal-oracle.mjs';
import {narrowingIds,expectedNarrowingObservation} from './narrowing-oracle.mjs';
import {nullableTransportIds,expectedNullableTransportObservation} from './nullable-transport-oracle.mjs';
import {arrayToStringIds,expectedArrayToStringObservation} from './array-to-string-oracle.mjs';
import {arrayJoinIds,expectedArrayJoinObservation} from './array-join-oracle.mjs';
import {expectedValueTypesObservation,valueTypesIds} from './value-types-oracle.mjs';
export {expectedValueTypesObservation} from './value-types-oracle.mjs';
import {expectedHtmlStringObservation,htmlStringIds} from './html-string-oracle.mjs';
export {expectedHtmlStringObservation} from './html-string-oracle.mjs';
import {expectedStringSubstrObservation,stringSubstrIds} from './string-substr-oracle.mjs';
export {expectedStringSubstrObservation} from './string-substr-oracle.mjs';
import {expectedSplitOwnershipObservation,splitOwnershipIds} from './split-ownership-oracle.mjs';
export {expectedSplitOwnershipObservation} from './split-ownership-oracle.mjs';
import {expectedTemplateStringObservation} from './template-string-oracle.mjs';
export {expectedTemplateStringObservation} from './template-string-oracle.mjs';
import {expectedQuotedStringLiteralObservation} from './quoted-string-literals-oracle.mjs';
export {expectedQuotedStringLiteralObservation} from './quoted-string-literals-oracle.mjs';
import {expectedStringWellFormedObservation} from './string-well-formed-oracle.mjs';
export {expectedStringWellFormedObservation} from './string-well-formed-oracle.mjs';
import {expectedNumberParseFloatObservation} from './number-parse-float-oracle.mjs';
export {expectedNumberParseFloatObservation} from './number-parse-float-oracle.mjs';
import {expectedNumberFormatObservation} from './number-format-oracle.mjs';
export {expectedNumberFormatObservation} from './number-format-oracle.mjs';
import {expectedIntToStringObservation} from './int-to-string-oracle.mjs';
export {expectedIntToStringObservation} from './int-to-string-oracle.mjs';
import {expectedPrimitiveIdentityObservation} from './primitive-identity-oracle.mjs';
export {expectedPrimitiveIdentityObservation} from './primitive-identity-oracle.mjs';
import {expectedStringSubstringObservation} from './string-substring-oracle.mjs';
export {expectedStringSubstringObservation} from './string-substring-oracle.mjs';
import {expectedStringConcatPaddingObservation} from './string-concat-padding-oracle.mjs';
export {expectedStringConcatPaddingObservation} from './string-concat-padding-oracle.mjs';
import {expectedConditionalObservation} from './conditional-oracle.mjs';
export {expectedConditionalObservation} from './conditional-oracle.mjs';
import {expectedCompoundValueObservation} from './compound-value-oracle.mjs';
export {expectedCompoundValueObservation} from './compound-value-oracle.mjs';
import {expectedAssignmentValueObservation} from './assignment-value-oracle.mjs';
export {expectedAssignmentValueObservation} from './assignment-value-oracle.mjs';
import {expectedLogicalAssignmentObservation} from './logical-assignment-oracle.mjs';
export {expectedLogicalAssignmentObservation} from './logical-assignment-oracle.mjs';
import {expectedNativeIntObservation} from './int-native-oracle.mjs';
export {expectedNativeIntObservation} from './int-native-oracle.mjs';
import {expectedUpdateObservation} from './update-oracle.mjs';
export {expectedUpdateObservation} from './update-oracle.mjs';
import {expectedCompoundObservation} from './compound-oracle.mjs';
export {expectedCompoundObservation} from './compound-oracle.mjs';
import {expectedAddObservation} from './string-add-oracle.mjs';
export {expectedAddObservation} from './string-add-oracle.mjs';
import {expectedTrimObservation} from './string-trim-oracle.mjs';
export {expectedTrimObservation} from './string-trim-oracle.mjs';
import {expectedStringLastIndexObservation} from './string-last-index-of-oracle.mjs';
export {expectedStringLastIndexObservation} from './string-last-index-of-oracle.mjs';
import {expectedLastIndexObservation} from './last-index-of-oracle.mjs';
export {expectedLastIndexObservation} from './last-index-of-oracle.mjs';
import {expectedToSplicedObservation} from './to-spliced-oracle.mjs';
export {expectedToSplicedObservation} from './to-spliced-oracle.mjs';
import {expectedCopyingObservation} from './copying-array-oracle.mjs';
export {expectedCopyingObservation} from './copying-array-oracle.mjs';
import {expectedRangeObservation} from './range-mutation-oracle.mjs';
export {expectedRangeObservation} from './range-mutation-oracle.mjs';
import {expectedHigherObservation} from './higher-order-oracle.mjs';
export {expectedHigherObservation} from './higher-order-oracle.mjs';
import {expectedDeclaredObservation} from './declared-value-oracle.mjs';
export {expectedDeclaredObservation} from './declared-value-oracle.mjs';
import {expectedConcatObservation} from './concat-oracle.mjs';
export {expectedConcatObservation} from './concat-oracle.mjs';
import {expectedForEach} from './for-each-oracle.mjs';
export {expectedForEach} from './for-each-oracle.mjs';
import {observeNumber} from './number-observation.mjs';
import {expectedReductionObservation} from './reduction-results.mjs';
export {expectedReductionObservation} from './reduction-results.mjs';
import {expectedNumberOperators} from './number-operators-oracle.mjs';
export {expectedNumberOperators} from './number-operators-oracle.mjs';
import {expectedExactMath} from './exact-math-oracle.mjs';
export {expectedExactMath} from './exact-math-oracle.mjs';
import {expectedConstructionObservation} from './string-construction-oracle.mjs';
export {expectedConstructionObservation} from './string-construction-oracle.mjs';
import {expectedArrayPredicateObservation} from './array-predicate-oracle.mjs';
export {expectedArrayPredicateObservation} from './array-predicate-oracle.mjs';
import {expectedFixedNumber} from './fixed-number-oracle.mjs';
export {expectedFixedNumber} from './fixed-number-oracle.mjs';
import {expectedMapFilterObservation} from './map-filter-oracle.mjs';
export {expectedMapFilterObservation} from './map-filter-oracle.mjs';
import {expectedTemporaryObservation} from './temporary-index-oracle.mjs';
export {expectedTemporaryObservation} from './temporary-index-oracle.mjs';
import {expectedSliceObservation} from './slice-oracle.mjs';
export {expectedSliceObservation} from './slice-oracle.mjs';
import {expectedMath} from './math-oracle.mjs';
export {expectedMath} from './math-oracle.mjs';
import {expectedPredicate} from './number-predicate-oracle.mjs';
import {expectedNumber} from './number-oracle.mjs';
export {expectedNumber} from './number-oracle.mjs';
// Independent expected-result calculations from the workload specifications.
// Linear series use closed forms, not the implementation's records/callbacks/arrays.
export function expectedChecksum(id, n, seed, inputText) {
 if(id==='numeric_string_conversion')return expectedNumericStringConversionObservation(id,n,seed,inputText).checksum;
 if(id==='scalar_unary_plus')return expectedScalarUnaryPlusObservation(id,n,seed,inputText).checksum;
 if(id==='string_concatenation')return expectedStringConcatenationObservation(id,n,seed,inputText).checksum;
 if(id==='array_callback_errors')return expectedArrayCallbackErrorsObservation(id,n,seed,inputText).checksum; if(id==='map_for_each')return expectedMapForEachObservation(id,n,seed,inputText).checksum; if(id==='set_relations')return expectedSetRelationsObservation(id,n,seed,inputText).checksum; if(id==='string_iteration')return expectedStringIterationObservation(id,n,seed,inputText).checksum; if(id==='comma_contexts')return expectedCommaContextsObservation(id,n,seed,inputText).checksum; if(id==='comma_expressions')return expectedCommaExpressionsObservation(id,n,seed,inputText).checksum; if(id==='string_indexing')return expectedStringIndexingObservation(id,n,seed,inputText).checksum; if(id==='optional_fields')return expectedOptionalFieldsObservation(id,n,seed,inputText).checksum; if(id==='interfaces')return expectedInterfacesObservation(id,n,seed,inputText).checksum; if(id==='type_aliases')return expectedTypeAliasesObservation(id,n,seed,inputText).checksum; if(id==='numeric_literals')return expectedNumericLiteralsObservation(id,n,seed,inputText).checksum; if(id==='map_group_by')return expectedMapGroupByObservation(id,n,seed,inputText).checksum; if(id==='iterable_call_spread')return expectedIterableCallSpreadObservation(id,n,seed,inputText).checksum; if(id==='iterable_array_spread')return expectedIterableArraySpreadObservation(id,n,seed,inputText).checksum; if(id==='array_from_collections')return expectedArrayFromCollectionsObservation(id,n,seed,inputText).checksum; if(id==='map_iteration')return expectedMapIterationObservation(id,n,seed,inputText).checksum; if(id==='set_algebra')return expectedSetAlgebraObservation(id,n,seed,inputText).checksum; if(id==='set_for_each')return expectedSetForEachObservation(id,n,seed,inputText).checksum; if(id==='set_entries')return expectedSetEntriesObservation(id,n,seed,inputText).checksum; if(id==='map_snapshots')return expectedMapSnapshotsObservation(id,n,seed,inputText).checksum; if(id==='map_values')return expectedMapsObservation(id,n,seed,inputText).checksum; if(id==='set_values')return expectedSetsObservation(id,n,seed,inputText).checksum; if(id==='enum_values')return expectedEnumsObservation(id,n,seed,inputText).checksum; if(id==='type_assertions')return expectedTypeAssertionsObservation(id,n,seed,inputText).checksum;
 if(id==='exception_completion')return expectedExceptionCompletionObservation(id,n,seed,inputText).checksum;
 if(id==='testing_expectation_predicates')return expectedTestingExpectationChecksum(n,seed);
 if(id==='testing_scalar_assertions')return expectedTestingAssertionChecksum(n,seed);
 if(arrayFromIds.includes(id))return expectedArrayFromObservation(id,n,seed,inputText).checksum;
 if(stringAtIds.includes(id))return expectedStringAtObservation(id,n,seed,inputText).checksum;
 if(numberParseIntIds.includes(id))return expectedNumberParseIntObservation(id,n,seed).checksum;
 if(numberToStringIds.includes(id))return expectedNumberToStringObservation(id,n,seed).checksum;
 if(objectIsIds.includes(id))return expectedObjectIsObservation(id,n,seed,inputText).checksum;
 if(mathF16roundIds.includes(id))return expectedMathF16roundObservation(id,n,seed).checksum;
 if(mathRandomIds.includes(id))return expectedMathRandomObservation(id,n,seed).checksum;
 if(stringFromCodesIds.includes(id))return expectedStringFromCodesObservation(id,n,seed).checksum;
 if(stringCodeAtIds.includes(id))return expectedStringCodeAtObservation(id,n,seed).checksum;
 if(unionArrayIds.includes(id))return expectedUnionArrayObservation(id,n,seed,inputText).checksum;
 if(objectValuesIds.includes(id))return expectedObjectValuesObservation(id,n,seed,inputText).checksum;
 if(destructuringIds.includes(id))return expectedDestructuringObservation(id,n,seed,inputText).checksum;
 if(callableExpressionsIds.includes(id))return expectedCallableExpressionsObservation(id,n,seed,inputText).checksum;
 if(labeledControlIds.includes(id))return expectedLabeledControlObservation(id,n,seed,inputText).checksum;
 if(inOperatorIds.includes(id))return expectedInOperatorObservation(id,n,seed,inputText).checksum;
 if(deferredInitializationIds.includes(id))return expectedDeferredInitializationObservation(id,n,seed,inputText).checksum;
 if(globalNumberNamesIds.includes(id))return expectedGlobalNumberNamesObservation(id,n,seed,inputText).checksum;
 if(functionExpressionsIds.includes(id))return expectedFunctionExpressionsObservation(id,n,seed,inputText).checksum;
 if(switchStatementsIds.includes(id))return expectedSwitchStatementsObservation(id,n,seed,inputText).checksum;
 if(bracelessBodiesIds.includes(id))return expectedBracelessBodiesObservation(id,n,seed,inputText).checksum;
 if(destructuringAssignmentIds.includes(id))return expectedDestructuringAssignmentObservation(id,n,seed,inputText).checksum;
 if(declarationListsIds.includes(id))return expectedDeclarationListsObservation(id,n,seed,inputText).checksum;
 if(forPatternInitializersIds.includes(id))return expectedForPatternInitializersObservation(id,n,seed,inputText).checksum;
 if(nestedRestPatternsIds.includes(id))return expectedNestedRestPatternsObservation(id,n,seed,inputText).checksum;
 if(destructuringRestIds.includes(id))return expectedDestructuringRestObservation(id,n,seed,inputText).checksum;
 if(objectHasOwnIds.includes(id))return expectedObjectHasOwnObservation(id,n,seed,inputText).checksum;
 if(destructuringDefaultsIds.includes(id))return expectedDestructuringDefaultsObservation(id,n,seed,inputText).checksum;
 if(borrowedOptionalFallbackIds.includes(id))return expectedBorrowedOptionalFallbackObservation(id,n,seed,inputText).checksum;
 if(dataUnionsIds.includes(id))return expectedDataUnionsObservation(id,n,seed,inputText).checksum;
 if(objectKeysIds.includes(id))return expectedObjectKeysObservation(id,n,seed,inputText).checksum;
 if(arrayEntriesIds.includes(id))return expectedArrayEntriesObservation(id,n,seed,inputText).checksum;
 if(tupleTypesIds.includes(id))return expectedTupleTypesObservation(id,n,seed,inputText).checksum;
 if(stringNormalizeIds.includes(id))return expectedStringNormalizeObservation(id,n,seed).checksum;
 if(arrayLocaleStringIds.includes(id))return expectedArrayLocaleStringObservation(id,n,seed,inputText).checksum;
 if(numberLocaleStringIds.includes(id))return expectedNumberLocaleStringObservation(id,n,seed).checksum;
 if(stringRawIds.includes(id))return expectedStringRawObservation(id,n,seed,inputText).checksum;
 if(stringLocaleCaseIds.includes(id))return expectedStringLocaleCaseObservation(id,n,seed).checksum;
 if(arrayKeysValuesIds.includes(id))return expectedArrayKeysValuesObservation(id,n,seed,inputText).checksum;
 if(arrayLiteralSpreadIds.includes(id))return expectedArrayLiteralSpreadObservation(id,n,seed,inputText).checksum;
 if (id === 'std_path_basename_ascii') {
  let digest = 0;
  for (let i = 0; i < n; i++) {
   const codes = (i + seed) % 2 === 0 ? [97,114,99,104,105,118,101] : [114,101,112,111,114,116];
   for (const code of [...codes, 255]) digest = (digest * 131 + code) % 1000000007;
  }
  return digest;
 }
 if(callSpreadIds.includes(id))return expectedCallSpreadObservation(id,n,seed,inputText).checksum;
 if(id==='logical_nullable_string_values')return expectedLogicalValueObservation(id,n,seed).checksum;
 if(restParameterIds.includes(id))return expectedRestParameterObservation(id,n,seed,inputText).checksum;
 if(optionalDefaultParameterIds.includes(id))return expectedOptionalDefaultParameterObservation(id,n,seed,inputText).checksum;
 if(nullableCallableIds.includes(id))return expectedNullableCallableObservation(id,n,seed,inputText).checksum;
 if(id==='generic_record_nested_int')return expectedCallableFoundation('generic_function_record_pipeline',n,seed);
 if(callableFoundationIds.includes(id))return expectedCallableFoundation(id,n,seed);
 if(/^array_find_(?:last_)?(?:int|number|boolean|string|nested|record|null|undefined)$/.test(id))return expectedFindObservation(id,n,seed,inputText).checksum;
 if(/^array_unshift_(?:int|number|boolean|string|nested|record|null|undefined)$/.test(id))return expectedUnshiftObservation(id,n,seed,inputText).checksum;
 if(/^array_splice_(?:int|number|boolean|string|nested|record|null|undefined)$/.test(id))return expectedSpliceObservation(id,n,seed,inputText).checksum;
 if(/^array_(?:sort|to_sorted)_(?:default|compare)_(?:int|number|boolean|string|nested|record|null|undefined)$/.test(id))return expectedSortingObservation(id,n,seed,inputText).checksum;
 if(/^array_(?:flat|flat_map)_(?:int|number|boolean|string|nested|record|null|undefined)$/.test(id))return expectedFlatObservation(id,n,seed,inputText).checksum;
 if(/^array_of_(?:int|number|boolean|string|nested|record|null|undefined)$/.test(id))return expectedOfObservation(id,n,seed,inputText).checksum;
 if(/^nullish_(?:(?:int|number|boolean|string|array|record|null_undefined)_value|nullable_fallback_int)$/.test(id))return expectedNullishObservation(id,n,seed,inputText).checksum;
 if(/^nullish_assign_(?:int|number|boolean|string|array|record|nullable_int|nullable_string)_value$/.test(id))return expectedNullishAssignmentObservation(id,n,seed,inputText).checksum;
 if(/^optional_chain_(?:int_field|number_field|boolean_field|string_length|array_index|array_join|record_field|nullable_chain)$/.test(id))return expectedOptionalChainObservation(id,n,seed,inputText).checksum;
 if(arrayAtIds.includes(id))return expectedArrayAtObservation(id,n,seed,inputText).checksum;
 if(arrayRemovalIds.includes(id))return expectedArrayRemovalObservation(id,n,seed,inputText).checksum;
 if(narrowingIds.includes(id))return expectedNarrowingObservation(id,n,seed,inputText).checksum;
 if(nullableTransportIds.includes(id))return expectedNullableTransportObservation(id,n,seed,inputText).checksum;
 if(arrayToStringIds.includes(id))return expectedArrayToStringObservation(id,n,seed,inputText).checksum;
 if(arrayJoinIds.includes(id))return expectedArrayJoinObservation(id,n,seed,inputText).checksum;
 if(valueTypesIds.includes(id))return expectedValueTypesObservation(id,n,seed,inputText).checksum;
 if(htmlStringIds.includes(id))return expectedHtmlStringObservation(id,n,seed,inputText).checksum;
 if(stringSubstrIds.includes(id))return expectedStringSubstrObservation(id,n,seed,inputText).checksum;
 if(splitOwnershipIds.includes(id))return expectedSplitOwnershipObservation(id,n,seed,inputText).checksum;
 if(id.startsWith('template_'))return expectedTemplateStringObservation(id,n,seed,inputText).checksum;
 if(id.startsWith('quoted_string_literals_'))return expectedQuotedStringLiteralObservation(id,n,seed,inputText).checksum;
 if(/^string_(is|to)_well_formed_/.test(id))return expectedStringWellFormedObservation(id,n,seed,inputText).checksum;
 if(id.startsWith('number_parse_float_'))return expectedNumberParseFloatObservation(id,n,seed,inputText).checksum;
 if(/^number_to_(fixed|exponential|precision)_/.test(id))return expectedNumberFormatObservation(id,n,seed,inputText).checksum;
 if(id.startsWith('int_to_string_'))return expectedIntToStringObservation(id,n,seed,inputText).checksum;
 if(/^(string_(to_string|value_of)_(ascii|unicode)|boolean_(to_string|value_of)|number_value_of|int_value_of)$/.test(id))return expectedPrimitiveIdentityObservation(id,n,seed,inputText).checksum;
 if(id.startsWith('string_substring_'))return expectedStringSubstringObservation(id,n,seed,inputText).checksum;
 if(/^string_(concat_(zero|variadic)_|pad_(start|end)_)/.test(id))return expectedStringConcatPaddingObservation(id,n,seed,inputText).checksum;
 if(id.startsWith('conditional_'))return expectedConditionalObservation(id,n,seed,inputText).checksum;
 if(id.startsWith('compound_value_'))return expectedCompoundValueObservation(id,n,seed).checksum;
 if(/^(int|number|boolean)_assignment_value$/.test(id))return expectedAssignmentValueObservation(id,n,seed).checksum;
 if(id.startsWith('boolean_logical_'))return expectedLogicalAssignmentObservation(id,n,seed).checksum;
 if(id.startsWith('int_native_'))return expectedNativeIntObservation(id,n,seed).checksum;
 if(id.startsWith('update_'))return expectedUpdateObservation(id,n,seed).checksum;
 if(id.startsWith('compound_'))return expectedCompoundObservation(id,n,seed,inputText).checksum;
 if(/^string_add_(direct|chained)_/.test(id))return expectedAddObservation(id,n,seed,inputText).checksum;
 if(/^string_trim_(start|end|left|right)_/.test(id))return expectedTrimObservation(id,n,seed,inputText).checksum;
 if(id.startsWith('string_last_index_of_'))return expectedStringLastIndexObservation(id,n,seed,inputText).checksum;
 if(id.startsWith('array_last_index_of_'))return expectedLastIndexObservation(id,n,seed,inputText).checksum;
 if(/^array_to_spliced_/.test(id))return expectedToSplicedObservation(id,n,seed,inputText).checksum;
 if(/^array_(to_reversed|with)_/.test(id))return expectedCopyingObservation(id,n,seed,inputText).checksum;
 if(/^array_(fill|copy_within)_/.test(id))return expectedRangeObservation(id,n,seed,inputText).checksum;
 if(id.startsWith('higher_'))return expectedHigherObservation(id,n,seed,inputText).checksum;
 if(id.startsWith('declared_'))return expectedDeclaredObservation(id,n,seed,inputText).checksum;
 if(id.startsWith('array_concat_'))return expectedConcatObservation(id,n,seed,inputText).checksum;
 if(id.startsWith('array_for_each_')){const value=expectedForEach(id,n,seed,inputText);return id==='array_for_each_number'||id==='array_for_each_nested_number'?observeNumber(value).checksum:value;}
 if(/^array_reduce(?:_right)?_/.test(id))return expectedReductionObservation(id,n,seed).checksum;
 if(/^number_(and|or|xor|not|shift_left|shift_right|shift_right_unsigned)$/.test(id))return expectedNumberOperators(id,n,seed).checksum;
 if(/^string_(repeat_|replace_(first|all)_|split_)/.test(id)) return expectedConstructionObservation(id,n,seed,inputText).checksum;
 if(/^array_(some|every|find_index|find_last_index)_/.test(id)) return expectedArrayPredicateObservation(id,n,seed,inputText).checksum;
 if(id==='number_direct_literal_index'||id==='number_fixed_local_operations') return expectedFixedNumber(id,n,seed).checksum;
 if(id.startsWith('array_map_')||id.startsWith('array_filter_')) return expectedMapFilterObservation(id,n,seed,inputText).checksum;
 if(id.startsWith('array_temporary_index_')) return expectedTemporaryObservation(id,n,seed,inputText).checksum;
 if(id.startsWith('array_slice_')) return expectedSliceObservation(id,n,seed,inputText).checksum;
 if(/^math_(round|sign|fround|min|max|clz32|imul)$/.test(id)) return expectedExactMath(id,n,seed).checksum;
 if(id.startsWith('math_')) return expectedMath(id,n,seed).checksum;
 if (/^number_is_(finite|nan|integer|safe_integer)$/.test(id)) return expectedPredicate(id,n,seed);
 if (id.startsWith('number_')) return expectedNumber(id,n,seed).checksum;
 const series = n * (n - 1) / 2;
 const linear = (coefficient, constant, initial = 0) => coefficient * series + n * constant + initial;
 switch (id) {
  case 'classic_for_control':
  case 'do_while_control':
  case 'else_if_control':
   return expectedControlFlowChecksum(id,n,seed);
  case 'int_divide':
  case 'int_negate':
  case 'int_less_equal':
  case 'int_greater_equal':
  case 'int_not_equal':
  case 'boolean_equal':
  case 'boolean_not_equal':
  case 'boolean_or':
  case 'boolean_short_circuit':
   return expectedPrimitive(id,n,seed);
  case 'string_starts_with_ascii':
  case 'string_ends_with_ascii':
  case 'string_includes_ascii':
  case 'string_index_of_ascii':
  case 'string_starts_with_unicode':
  case 'string_ends_with_unicode':
  case 'string_includes_unicode':
  case 'string_index_of_unicode':
   return expectedStringSearchChecksum(id,n,seed,inputText);
  case 'fixed_int_array_reverse': case 'growable_int_array_reverse': case 'owned_string_array_reverse':
   return expectedReverseChecksum(id,n,seed,inputText);
  case 'literal_ascii_uppercase': case 'dynamic_unicode_uppercase':
   return expectedUppercaseChecksum(id,n,seed,inputText);
  case 'string_equality': case 'string_ordering_ascii':
   return expectedStringComparison(id,n,seed,inputText);
  case 'string_char_at_unicode_middle_literal':
   if (inputText !== undefined) throw new Error('Literal midpoint operation has no external input');
   return expectedLiteralMiddleChecksum(n, seed);
  case 'string_construct_slice_unicode_long': case 'string_construct_slice_ascii_long':
   return expectedConstructionChecksum(id, n, seed, inputText);
  case 'string_length': case 'string_slice_ascii': case 'string_slice_unicode': case 'string_char_at':
  case 'string_char_at_ascii_long': case 'string_char_at_unicode_tail': case 'string_char_at_unicode_late': case 'string_char_at_unicode_middle': {
   const scalars = unicodeScalars(inputText), boundaries = byteBoundaries(scalars), bytes = boundaries.at(-1);
   if ((id === 'string_slice_ascii' || id === 'string_char_at_ascii_long') && scalars.some(code => code > 127)) throw new Error('ASCII input required');
   let first = 0, last = scalars.length;
   while (first < last && rustWhitespace(scalars[first])) first++;
   while (last > first && rustWhitespace(scalars[last - 1])) last--;
   let digest = id === 'string_length' ? seed : 0;
   const bound = offset => offset < 0 ? Math.max(0, bytes + offset) : Math.min(bytes, offset);
   for (let i = 0; i < n; i++) {
    if (id === 'string_length') { digest = (digest * 131 + ((i + seed) % 2 === 0 ? bytes : boundaries[last] - boundaries[first])) % 1000000007; continue; }
    let start, end;
    if (id.startsWith('string_char_at')) {
     start = id === 'string_char_at_unicode_tail' ? (i * 13 + seed) % 37 - 7
      : id === 'string_char_at_unicode_late' ? scalars.length - 7 + (i * 13 + seed) % 11
      : id === 'string_char_at_unicode_middle' ? Math.floor(scalars.length / 2) - 8 + (i * 13 + seed) % 17
      : (i * 13 + seed) % (scalars.length * 2 + 3) - scalars.length;
     if (start >= 0 && start < scalars.length) digest = (digest * 131 + scalars[start]) % 1000000007;
    } else {
     if (id === 'string_slice_unicode') {
      const coordinate = ordinal => ordinal < 0 ? boundaries[Math.max(0, scalars.length + ordinal)] - bytes : boundaries[Math.min(scalars.length, ordinal)];
      start = bound(coordinate((i * 13 + seed) % (scalars.length * 3 + 1) - scalars.length));
      end = bound(coordinate((i * 7 + seed) % (scalars.length * 3 + 1) - scalars.length));
     } else { start = bound((i * 13 + seed) % (bytes * 3 + 1) - bytes); end = bound((i * 7 + seed) % (bytes * 3 + 1) - bytes); }
     if (end > start) for (let position = 0; position < scalars.length; position++) if (boundaries[position] >= start && boundaries[position + 1] <= end) digest = (digest * 131 + scalars[position]) % 1000000007;
    }
    digest = (digest * 131 + 255) % 1000000007;
   }
   return digest;
  }
  case 'scalar_control': {
   let state = seed, sum = seed;
   for (let i = 0; i < n; i++) { state = (17 * state + 13) % 1009; sum += state > 100 && state % 2 === 0 ? state : -1; }
   return sum;
  }
  case 'ascii_strings': {
   let result = 0;
   for (let i = 0; i < n; i++) {
    const codes = (i + seed) % 2 === 0 ? [115, 97, 109] : [112, 97, 116];
    for (const code of [...codes, 255]) result = (result * 131 + code) % 1000000007;
   } return result;
  }
  case 'record_mutation': {
   const first = seed + 1 + ((seed + 1) % 2), last = seed + n - ((seed + n) % 2);
   return first > last ? 0 : ((last - first) / 2 + 1) * (first + last) / 2;
  }
  case 'deep_clone': return linear(2, seed + 3, seed);
  case 'scalar_arrays': {
   let total = seed;
   for (let i = 0; i < n; i++) {
    for (let position = 0; position < 32; position++) {
     const value = position === 3 ? i : seed + i + position;
     if (value === seed) continue;
     if (value > 20000) break;
     total += value;
    }
    if (i % 2 === 0) total++;
   } return total;
  }
  case 'owned_arrays': return linear(4, seed + 7, seed);
  case 'nested_arrays': return linear(3, 2 * seed + 6, seed);
  case 'record_arrays': return linear(4, seed + 3, seed);
  case 'field_restore': return linear(4, seed + 4, seed);
  case 'local_closures': return linear(1, seed + 2, seed);
  case 'closure_factory': return linear(1, seed + 1);
  case 'callback_contracts': return linear(1, 2 * seed);
  case 'owned_callbacks': return linear(2, seed + 3, seed);
  case 'callable_replacement': return linear(1, seed + 2);
  case 'recursive_data': return linear(2, 2 * seed + 37);
  case 'recursive_callbacks': return linear(5, 9 * seed + 51);
  case 'recursive_factory': return linear(2, seed + 3);
  case 'ordered_removal': {
   let weightedIndices = 0;
   // Surviving original indices are 1, 2, 4, ..., 30 in that order.
   for (let position = 1; position <= 29; position++) weightedIndices += position * (position < 3 ? position : position + 1);
   return linear(440, 439 * seed + 34 + weightedIndices);
  }
  case 'owned_arguments': return linear(4, 2 * seed, seed);
  case 'capture_replacement': return linear(4, 5 * seed + 2, seed);
  case 'recursive_clone': {
   let digest = 0;
   for (const original of [false, true]) for (let index = 0; index < n; index++) {
    const depth = (index + seed) % 4, width = (index + seed) % 5, values = [];
    for (let level = depth - 1; level >= 0; level--) values.push((index + level + seed) % 1009);
    values.push(index % 1009);
    for (let child = 0; child < width; child++) values.push((3 * index + child + seed) % 1009);
    if (original) { values[0] = index + 2000; if (values.length > 1) values[1] = seed + 3000; }
    for (const value of values) digest = (digest * 131 + value) % 1000000007;
    digest = (digest * 131 + 255) % 1000000007;
   } return digest;
  }
  case 'retained_numbers': {
   let digest = 0;
   for (let position = 0; position < n; position++) digest = (digest * 131 + (position * 17 + seed) % 1009) % 1000000007;
   return digest;
  }
  case 'retained_rows': {
   let digest = 0;
   for (let row = 0; row < n; row++) {
    for (let column = 0; column < (row + seed) % 65; column++) digest = (digest * 131 + (row * 3 + seed + column) % 1009) % 1000000007;
    digest = (digest * 131 + 255) % 1000000007;
   } return digest;
  }
  case 'array_churn': {
   let total = seed;
   for (let cycle = 0; cycle < n; cycle++) {
    const width = (cycle + seed) % 65, indices = Array.from({length: width}, (_, index) => index);
    let count = 0;
    while (indices.length > Math.floor(width / 2)) {
     const at = (cycle + seed + count) % indices.length;
     total += (cycle + seed + indices[at]) % 1009;
     indices.splice(at, 1); count++;
    }
    for (let position = 0; position < indices.length; position++) total += ((cycle + seed + indices[position]) % 1009) * (position + 1);
    total += seed * (indices.length + 1) + cycle * (indices.length + 2);
    for (let original = 0; original < width; original++) total += original === 0 ? cycle : (cycle + seed + original) % 1009;
   } return total;
  }
  case 'record_array_churn': {
   let total = seed;
   for (let cycle = 0; cycle < n; cycle++) {
    const width = (cycle + seed) % 17;
    total += 2 * width * (3 * cycle + 2 * seed) + 2 * width * (width - 1) + (width ? seed - cycle : 0);
   } return total;
  }
  case 'literal_normalization': {
   let digest = 0;
   for (let position = 0; position < n; position++) {
    const expected = `item ${String((position + seed) % 256).padStart(4, '0')} abcxyz`;
    for (let byte = 0; byte < expected.length; byte++) digest = (digest * 131 + expected.charCodeAt(byte)) % 1000000007;
    digest = (digest * 131 + 255) % 1000000007;
   } return digest;
  }
  case 'dynamic_string_clone': {
   const inputScalars = unicodeScalars(inputText);
   let digest = 0;
   for (let position = 0; position < n; position++) {
    for (const scalar of inputScalars) digest = (digest * 131 + scalar) % 1000000007;
    digest = (digest * 131 + 255) % 1000000007;
   } return digest;
  }
  case 'dynamic_normalization': {
   const codes = unicodeScalars(inputText);
   let first = 0, last = codes.length;
   while (first < last && rustWhitespace(codes[first])) first++;
   while (last > first && rustWhitespace(codes[last - 1])) last--;
   const expectedCodes = codes.slice(first,last).map(code => code >= 65 && code <= 90 ? code + 32 : code);
   if (expectedCodes.some(code => code > 127 && code !== 0xfeff && !rustWhitespace(code))) throw new Error("Normalization oracle requires ASCII case mappings");
   let digest = 0;

   for (let position = 0; position < n; position++) {
    for (const code of expectedCodes) digest = (digest * 131 + code) % 1000000007;
    digest = (digest * 131 + 255) % 1000000007;
   } return digest;
  }
  case 'recursive_growth': {
   let total = seed;
   for (let cycle = 0; cycle < n; cycle++) { const width = (cycle + seed) % 33; total += cycle + width * seed + width * (width - 1) / 2; }
   return total;
  }
  case 'unit_string_array_includes': case 'unit_string_array_index_of': {
   const scalars = unicodeScalars(inputText);
   if (!scalars.length) throw new Error('Scalar search requires nonempty string input');
   const positions = new Map();
   for (let position = 0; position < 129; position++) {
    const scalar = scalars[(position * 37 + seed) % scalars.length];
    if (!positions.has(scalar)) positions.set(scalar, []);
    positions.get(scalar).push(position);
   }
   let sum = seed;
   for (let index = 0; index < n; index++) {
    const choice = (index + seed) % 4;
    const needle = choice === 0 ? 65 : choice === 1 ? 0x1f600 : choice === 2 ? scalars[(index * 17 + seed) % (scalars.length + 2)] : undefined;
    const from = (index * 13 + seed) % 387 - 193;
    const start = Math.max(0, Math.min(129, from < 0 ? 129 + from : from));
    const found = positions.get(needle)?.find(position => position >= start) ?? -1;
    sum += id.endsWith('_includes') ? Number(found >= 0) : found;
   }
   return sum;
  }
  case 'dynamic_string_array_includes': case 'dynamic_string_array_index_of': {
   if (inputText !== '  Label Å  ') throw new Error('Dynamic search input specification changed');
   const includes = id.endsWith('_includes'), width = 129;
   const rare = (((100 - seed) % 101 + 101) % 101) * 71 % 101;
   let sum = seed;
   for (let i = 0; i < n; i++) {
    const from = (i * 13 + seed) % 387 - 193;
    const start = Math.max(0, Math.min(width, from < 0 ? width + from : from));
    const choice = (i + seed) % 3;
    let at = -1;
    if (choice === 0) {
     const first = start === rare || start === rare + 101 ? start + 1 : start;
     if (first < width) at = first;
    } else if (choice === 1) {
     const first = rare < start ? rare + 101 : rare;
     if (first >= start && first < width) at = first;
    }
    sum += includes ? Number(at >= 0) : at;
   }
   return sum;
  }
  case 'int_array_includes': case 'int_array_index_of':
  case 'boolean_array_includes': case 'boolean_array_index_of':
  case 'string_array_includes': case 'string_array_index_of': {
   const width = 129, includes = id.endsWith('_includes');
   if (id.startsWith('string_') && inputText !== 'label 064 Å') throw new Error('Search input specification changed');
   const positiveMod = (value, modulus) => ((value % modulus) + modulus) % modulus;
   let sum = seed;
   for (let i = 0; i < n; i++) {
    const from = (i * 13 + seed) % 387 - 193;
    const start = Math.max(0, Math.min(width, from < 0 ? width + from : from));
    let at = -1;
    if (id.startsWith('int_')) {
     const needle = (i * 17 + seed) % 137;
     // Inverse of 37 modulo 101 is 71. Locate both possible duplicates
     // algebraically; do not call JavaScript search or construct the array.
     if (needle < 101) {
      let first = positiveMod(needle - seed, 101) * 71 % 101;
      if (first < start) first += 101;
      if (first >= start && first < width) at = first;
     }
    } else if (id.startsWith('boolean_')) {
     const falseAt = positiveMod(100 - seed, 101) * 71 % 101;
     if ((i + seed) % 2 === 0) {
      const first = start === falseAt || start === falseAt + 101 ? start + 1 : start;
      if (first < width) at = first;
     } else {
      const first = falseAt >= start ? falseAt : falseAt + 101;
      if (first >= start && first < width) at = first;
     }
    } else {
     const ordinal = (i * 17 + seed) % 137;
     const first = ordinal < width ? ordinal : 64;
     if (first >= start && first < width) at = first;
    }
    sum += includes ? Number(at >= 0) : at;
   }
   return sum;
  }
  default: throw new Error(`No independent oracle for ${id}`);
 }
}

// Numeric scalar and UTF8-byte specifications are independent of measured helpers.
export function unicodeScalars(input) {
 if (typeof input !== 'string') throw new Error('Missing string input');
 const result = [];
 for (const scalar of input) {
  const code = scalar.codePointAt(0);
  if (code >= 0xd800 && code <= 0xdfff) throw new Error('Input is not valid Unicode');
  result.push(code);
 } return result;
}
export function byteBoundaries(scalars) {
 const result = [0];
 for (const code of scalars) result.push(result.at(-1) + (code < 0x80 ? 1 : code < 0x800 ? 2 : code < 0x10000 ? 3 : 4));
 return result;
}
export function rustWhitespace(code) {
 return [9,10,11,12,13,32,133,160,5760,8232,8233,8239,8287,12288].includes(code) || code >= 8192 && code <= 8202;
}
export function constructionSpecificationScalars(id, inputText) {
 const unicode = id === 'string_construct_slice_unicode_long';
 if (!unicode && id !== 'string_construct_slice_ascii_long') throw new Error('Unknown construction ID');
 const expected = [65,66,67,68,69,70,71,72], block = unicode ? [65,0x1f600,233,20013,931] : [65,122,48,57,33,63,65,66,67,120,121,122];
 for (let repeat = 0; repeat < 256; repeat++) expected.push(...block);
 expected.push(97,98,99,100,101,102,103,104);
 const actual = unicodeScalars(inputText);
 if (actual.length !== expected.length || actual.some((code,index) => code !== expected[index])) throw new Error('Construction input specification changed');
 return expected;
}
export function expectedConstructionChecksum(id, n, seed, inputText) {
 if (!Number.isSafeInteger(n) || n < 0 || n > 2000 || !Number.isSafeInteger(seed) || seed < 0 || seed > 1000000) throw new Error('Construction resource/integer range exceeded');
 const scalars = constructionSpecificationScalars(id,inputText), boundaries = byteBoundaries(scalars), bytes = boundaries.at(-1);
 let digest = 0;
 for (let i = 0; i < n; i++) {
  const first = (i + seed) % 8, last = bytes - 1 - (i * 7 + seed) % 8;
  for (let position = 0; position < scalars.length; position++) if (boundaries[position] >= first && boundaries[position + 1] <= last) digest = (digest * 131 + scalars[position]) % 1000000007;
  digest = (digest * 131 + 255) % 1000000007;
 } return digest;
}
export function literalMiddleScalars() {
 const scalars = [];
 for (let i = 0; i < 8192; i++) scalars.push(65,0x1f600,233,90);
 scalars.push(0x1f600,90);return scalars;
}
export function expectedLiteralMiddleChecksum(n, seed) {
 if (!Number.isSafeInteger(n) || n < 0 || n > 2048) throw new Error('Literal midpoint count must be 0..2048');
 if (!Number.isSafeInteger(seed) || seed < 0 || seed > 1000000) throw new Error('Literal midpoint seed must be 0..1000000');
 const scalars = literalMiddleScalars(), middle = Math.floor(scalars.length / 2);
 let digest = 0;
 for (let i = 0; i < n; i++) { digest = (digest * 131 + scalars[middle - 8 + (i * 13 + seed) % 17]) % 1000000007; digest = (digest * 131 + 255) % 1000000007; }
 return digest;
}

// Compare independent scalar/ASCII sequences rather than the measured string operators.
function sequenceOrder(left, right) {
    const length = Math.min(left.length, right.length);
    for (let i = 0; i < length; i++) {
        if (left[i] < right[i]) return -1;
        if (left[i] > right[i]) return 1;
    }
    return Math.sign(left.length - right.length);
}

export function expectedStringComparison(id, n, seed, input) {
    if (!Number.isSafeInteger(n) || n < 0 || n > 1000000 || !Number.isSafeInteger(seed) || seed < 0 || seed > 100) throw new Error('bounded nonnegative inputs required');
    const equality = id === 'string_equality';
    if (!equality && id !== 'string_ordering_ascii') throw new Error('unknown workload');
    const texts = equality ? ['', 'alpha', 'alpha\0', 'é', 'e\u0301', '😀', input] : ['', 'a', 'aa', 'alpha', 'alphabet', 'z', input];
    const sequences = texts.map(unicodeScalars);
    if (sequences.some(sequence => sequence.some(code => code >= 0xD800 && code <= 0xDFFF))) throw new Error('valid Unicode input required');
    if (!equality && sequences.some(sequence => sequence.some(code => code > 127))) throw new Error('ASCII ordering input required');
    let sum = seed;
    for (let i = 0; i < n; i++) {
        const left = (13 * i + seed) % 7, right = (17 * i + 3 * seed) % 7;
        const order = (a, b) => sequenceOrder(sequences[a % 7], sequences[b % 7]);
        if (equality) {
            if (order(left, right) === 0) sum++;
            if (order(left + 1, right + 2) !== 0) sum += 2;
        } else {
            if (order(left, right) < 0) sum++;
            if (order(left + 1, right + 2) <= 0) sum += 2;
            if (order(left + 2, right + 3) > 0) sum += 4;
            if (order(left + 3, right + 4) >= 0) sum += 8;
        }
    }
    return sum;
}

// Explicit stable uppercase domain, independent of measured uppercase methods.
// Explicit shared Unicode mappings: never call either measured uppercase API.
const expansions = new Map([[0xDF,[83,83]],[0xFB00,[70,70]],[0xFB01,[70,73]],[0xFB02,[70,76]],[0xE9,[0xC9]]]);
function uppercaseScalars(text) {
 if(typeof text !== 'string') throw new Error('String input required');
 const result=[];
 for(const scalar of text) {
  const code=scalar.codePointAt(0);
  if(code>=0xD800&&code<=0xDFFF) throw new Error('Valid Unicode required');
  if(code>=97&&code<=122) result.push(code-32);
  else if(expansions.has(code)) result.push(...expansions.get(code));
  else if(code<=127 || code===0x301 || code===0x1F600 || code===0xC9) result.push(code);
  else throw new Error('Input outside explicit shared uppercase mapping domain');
 }
 return result;
}
export function expectedUppercaseChecksum(id,n,seed,input) {
 if(!Number.isSafeInteger(n)||n<0||n>1000000||!Number.isSafeInteger(seed)||seed<0||seed>100) throw new Error('Bounded nonnegative inputs required');
 let outputs;
 if(id==='literal_ascii_uppercase') {
  if(input!==undefined) throw new Error('Literal operation has no external input');
  outputs=[
   [65,66,67,32,88,89,90,32,48,57],
   [65,76,82,69,65,68,89,32,85,80,80,69,82],
   [32,78,85,76,0,32,76,79,87,69,82,32],
  ];
 } else if(id==='dynamic_unicode_uppercase') outputs=[uppercaseScalars(input)];
 else throw new Error('Unknown uppercase operation');
 let sum=0;
 for(let i=0;i<n;i++) {
  for(const code of outputs[(i+seed)%outputs.length]) sum=(sum*131+code)%1000000007;
  sum=(sum*131+255)%1000000007;
 }
 return sum;
}

export function expectedReverseChecksum(id,n,seed,input) {
 if(!Number.isSafeInteger(n)||n<0||n>1000000||!Number.isSafeInteger(seed)||seed<0||seed>100) throw new Error('Bounded reverse inputs required');
 if(id==='owned_string_array_reverse') {
  const inputScalars=unicodeScalars(input);
  const labels=[inputScalars,[201],[111,109,101,103,97],[0x1f600],[65,76,80,72,65]],codes=[17,23,31,41,53];
  let sum=seed;
  for(let cycle=0;cycle<n;cycle++) for(let position=0;position<5;position++) {
   const value=labels[cycle%2===0?4-position:position];
   let code=0;
   for(let label=0;label<5;label++) if(value.length===labels[label].length&&value.every((scalar,index)=>scalar===labels[label][index])) { code=codes[label]; break; }
   sum=(sum*131+code)%1000000007;
  } return sum;
 }
 if(id!=='fixed_int_array_reverse'&&id!=='growable_int_array_reverse') throw new Error('Unknown reverse operation');
 if(input!==undefined) throw new Error('Numeric reverse has no external input');
 let sum=seed;
 for(let cycle=0;cycle<n;cycle++) for(let position=0;position<9;position++) sum=(sum*131+seed+cycle+8-position)%1000000007;
 return sum;
}

// Byte sequence reference: no measured string search API or counterpart mapping.
function referenceUTF8(text) {
 const scalars=unicodeScalars(text), bytes=[], boundaries=[0];
 for(const code of scalars) {
  if(code<128) bytes.push(code);
  else if(code<2048) bytes.push(192+(code>>6),128+(code&63));
  else if(code<65536) bytes.push(224+(code>>12),128+((code>>6)&63),128+(code&63));
  else bytes.push(240+(code>>18),128+((code>>12)&63),128+((code>>6)&63),128+(code&63));
  boundaries.push(bytes.length);
 }
 return {scalars,bytes,boundaries};
}
export function expectedStringSearchResult(method,input,needle,position) {
 if(!['starts_with','ends_with','includes','index_of'].includes(method)) throw new Error('Unknown string search method');
 if(position!==undefined&&!Number.isSafeInteger(position)) throw new Error('Safe integer position required');
 const hay=referenceUTF8(input), sought=referenceUTF8(needle);
 const at=position===undefined?(method==='ends_with'?hay.bytes.length:0):Math.max(0,Math.min(hay.bytes.length,position));
 if(!hay.boundaries.includes(at)) throw new Error('UTF-8 boundary required');
 const equalAt=start=>start>=0&&start+sought.bytes.length<=hay.bytes.length&&sought.bytes.every((byte,index)=>byte===hay.bytes[start+index]);
 if(method==='starts_with') return equalAt(at);
 if(method==='ends_with') return sought.bytes.length<=at&&equalAt(at-sought.bytes.length);
 let found=-1;
 for(let start=at;start+sought.bytes.length<=hay.bytes.length;start++) if(equalAt(start)) {found=start;break;}
 return method==='index_of'?found:found>=0;
}
export function expectedStringSearchChecksum(id,n,seed,input) {
 const match=/^string_(starts_with|ends_with|includes|index_of)_(ascii|unicode)$/.exec(id);
 if(!match) throw new Error('Unknown string search workload');
 if(!Number.isSafeInteger(n)||n<0||n>1000000||!Number.isSafeInteger(seed)||seed<0||seed>100) throw new Error('Bounded search inputs required');
 const [,method,kind]=match, hay=referenceUTF8(input);
 if(kind==='ascii'&&hay.scalars.some(code=>code>127)) throw new Error('ASCII input required');
 const positions=[-7,0,1,3,7,8,9,11,15,16,18,22,23,999];
 for(const position of positions) if(!hay.boundaries.includes(Math.max(0,Math.min(hay.bytes.length,position)))) throw new Error('Input lacks queried UTF-8 boundary');
 const needles=kind==='ascii'?['','A','alpha','Z','missing\u0000','Aalpha','a']:['','A','é','😀','e\u0301','absent\u0000','é😀'];
 let sum=seed;
 for(let i=0;i<n;i++) {
  const needle=needles[(i*17+seed)%7], position=(i+seed)%2===0?undefined:positions[(i*13+seed)%14];
  const result=expectedStringSearchResult(method,input,needle,position);
  sum=(sum*131+(method==='index_of'?result+1:Number(result)))%1000000007;
 }
 return sum;
}

export function expectedControlFlowChecksum(id,n,seed) {
 if(!Number.isSafeInteger(n)||n<0||n>1000000||!Number.isSafeInteger(seed)||seed<0||seed>100)throw new Error('Bounded nonnegative inputs required');
 if(!['classic_for_control','do_while_control','else_if_control'].includes(id))throw new Error('Unknown control workload');
 let digest=seed,state=seed;
 const append=token=>{digest=(digest*131+token)%1000000007;};
 const branchCodes=[11,11,11,23,23,23,23,37,37,37,37,49,49];
 for(let cycle=0;cycle<n;cycle++) {
  if(id==='else_if_control') {
   state=(state*17+branchCodes[(state+cycle)%13]+seed)%997;append(state);continue;
  }
  const bound=(cycle+seed)%10,stop=(cycle*3+seed)%7;
  const maximumVisits=id==='classic_for_control'?bound:Math.max(1,bound);
  const visited=Math.min(maximumVisits,stop+1);
  for(const index of Array.from({length:visited},(_,ordinal)=>ordinal)) {
   append((index+seed)%97);
   if(index!==stop&&(index+seed)%3!==0)append((index*3+1)%97);
  }
  const broke=stop<maximumVisits;
  append(id==='classic_for_control'&&broke?stop:visited);
 }
 return digest;
}

// Independent exact-integer calculations from the draft specifications.
// Boolean outcomes and RHS schedules come from explicit truth tables;
// integer comparisons use the sign of a BigInt difference.
const modulus = 1000000007n;
const truth = {
 boolean_equal: [1n, 0n, 0n, 1n],
 boolean_not_equal: [0n, 1n, 1n, 0n],
 boolean_or: [0n, 1n, 1n, 1n],
 boolean_short_circuit: [0n, 1n, 1n, 3n],
};
const rhsTags = [1n, 1n, 2n, 2n];
export function expectedPrimitive(id, n, seed) {
 if (![n, seed].every(value => Number.isSafeInteger(value) && value >= 0 && value <= 1000000)) {
  throw new Error('Primitive domain requires n and seed in 0..1000000');
 }
 const names = ['int_divide', 'int_negate', 'int_less_equal', 'int_greater_equal', 'int_not_equal', ...Object.keys(truth)];
 if (!names.includes(id)) throw new Error(`Unknown primitive workload ${id}`);
 const iterations = BigInt(n), initial = BigInt(seed);
 let digest = initial, trace = 0n;
 for (let ordinal = 0n; ordinal < iterations; ordinal++) {
  let observed;
  if (truth[id]) {
   const code = Number((ordinal * 13n + initial) % 4n);
   observed = truth[id][code];
   if (id === 'boolean_short_circuit') trace = (trace * 131n + rhsTags[code]) % modulus;
  } else {
   const numerator = (ordinal * 37n + initial) % 2001n - 1000n;
   if (id === 'int_divide') {
    const generated = (ordinal * 17n + initial) % 127n - 63n;
    const divisor = generated === 0n ? 1n : generated;
    observed = numerator / divisor + 1100n;
   } else if (id === 'int_negate') {
    observed = 1100n - numerator;
   } else {
    const other = (ordinal * 17n + initial * 3n) % 2001n - 1000n;
    const difference = numerator - other;
    const sign = difference < 0n ? -1 : difference > 0n ? 1 : 0;
    const answer = id === 'int_less_equal' ? sign !== 1
      : id === 'int_greater_equal' ? sign !== -1 : sign !== 0;
    observed = answer ? 1n : 0n;
   }
  }
  digest = (digest * 131n + observed) % modulus;
 }
 if (id === 'boolean_short_circuit') digest = (digest * 131n + trace + iterations) % modulus;
 const result = Number(digest);
 if (!Number.isSafeInteger(result)) throw new Error('Oracle output not exactly representable');
 return result;
}

export function expectedNumberResult(id,n,seed) { return id.startsWith('higher_')?expectedHigherObservation(id,n,seed): id.startsWith('declared_')?expectedDeclaredObservation(id,n,seed): id.startsWith('array_for_each_')?observeNumber(expectedForEach(id,n,seed)): /^number_(and|or|xor|not|shift_left|shift_right|shift_right_unsigned)$/.test(id)?expectedNumberOperators(id,n,seed): id==='number_direct_literal_index'||id==='number_fixed_local_operations' ? expectedFixedNumber(id,n,seed) : id.startsWith('array_temporary_index_') ? expectedTemporaryObservation(id,n,seed) : /^math_(round|sign|fround|min|max|clz32|imul)$/.test(id) ? expectedExactMath(id,n,seed) : id.startsWith('math_') ? expectedMath(id,n,seed) : expectedNumber(id,n,seed); }

export function expectedExactResult(id,n,seed,inputText) { if(id==='numeric_string_conversion')return expectedNumericStringConversionObservation(id,n,seed,inputText); if(id==='scalar_unary_plus')return expectedScalarUnaryPlusObservation(id,n,seed,inputText); if(id==='string_concatenation')return expectedStringConcatenationObservation(id,n,seed,inputText); if(id==='array_callback_errors')return expectedArrayCallbackErrorsObservation(id,n,seed,inputText); if(id==='map_for_each')return expectedMapForEachObservation(id,n,seed,inputText); if(id==='set_relations')return expectedSetRelationsObservation(id,n,seed,inputText); if(id==='string_iteration')return expectedStringIterationObservation(id,n,seed,inputText); if(id==='comma_contexts')return expectedCommaContextsObservation(id,n,seed,inputText); if(id==='comma_expressions')return expectedCommaExpressionsObservation(id,n,seed,inputText); if(id==='string_indexing')return expectedStringIndexingObservation(id,n,seed,inputText); if(id==='optional_fields')return expectedOptionalFieldsObservation(id,n,seed,inputText); if(id==='interfaces')return expectedInterfacesObservation(id,n,seed,inputText); if(id==='type_aliases')return expectedTypeAliasesObservation(id,n,seed,inputText); if(id==='numeric_literals')return expectedNumericLiteralsObservation(id,n,seed,inputText); if(id==='map_group_by')return expectedMapGroupByObservation(id,n,seed,inputText); if(id==='iterable_call_spread')return expectedIterableCallSpreadObservation(id,n,seed,inputText); if(id==='iterable_array_spread')return expectedIterableArraySpreadObservation(id,n,seed,inputText); if(id==='array_from_collections')return expectedArrayFromCollectionsObservation(id,n,seed,inputText); if(id==='map_iteration')return expectedMapIterationObservation(id,n,seed,inputText); if(id==='set_algebra')return expectedSetAlgebraObservation(id,n,seed,inputText); if(id==='set_for_each')return expectedSetForEachObservation(id,n,seed,inputText); if(id==='set_entries')return expectedSetEntriesObservation(id,n,seed,inputText); if(id==='map_snapshots')return expectedMapSnapshotsObservation(id,n,seed,inputText); if(id==='map_values')return expectedMapsObservation(id,n,seed,inputText); if(id==='set_values')return expectedSetsObservation(id,n,seed,inputText); if(id==='enum_values')return expectedEnumsObservation(id,n,seed,inputText); if(id==='type_assertions')return expectedTypeAssertionsObservation(id,n,seed,inputText); if(id==='exception_completion')return expectedExceptionCompletionObservation(id,n,seed,inputText); return callableExpressionsIds.includes(id)?expectedCallableExpressionsObservation(id,n,seed,inputText):labeledControlIds.includes(id)?expectedLabeledControlObservation(id,n,seed,inputText):inOperatorIds.includes(id)?expectedInOperatorObservation(id,n,seed,inputText):deferredInitializationIds.includes(id)?expectedDeferredInitializationObservation(id,n,seed,inputText):globalNumberNamesIds.includes(id)?expectedGlobalNumberNamesObservation(id,n,seed,inputText):functionExpressionsIds.includes(id)?expectedFunctionExpressionsObservation(id,n,seed,inputText):switchStatementsIds.includes(id)?expectedSwitchStatementsObservation(id,n,seed,inputText):bracelessBodiesIds.includes(id)?expectedBracelessBodiesObservation(id,n,seed,inputText):destructuringAssignmentIds.includes(id)?expectedDestructuringAssignmentObservation(id,n,seed,inputText):declarationListsIds.includes(id)?expectedDeclarationListsObservation(id,n,seed,inputText):forPatternInitializersIds.includes(id)?expectedForPatternInitializersObservation(id,n,seed,inputText):nestedRestPatternsIds.includes(id)?expectedNestedRestPatternsObservation(id,n,seed,inputText):destructuringRestIds.includes(id)?expectedDestructuringRestObservation(id,n,seed,inputText):objectHasOwnIds.includes(id)?expectedObjectHasOwnObservation(id,n,seed,inputText):unionArrayIds.includes(id)?expectedUnionArrayObservation(id,n,seed,inputText):objectValuesIds.includes(id)?expectedObjectValuesObservation(id,n,seed,inputText):destructuringIds.includes(id)?expectedDestructuringObservation(id,n,seed,inputText):destructuringDefaultsIds.includes(id)?expectedDestructuringDefaultsObservation(id,n,seed,inputText):borrowedOptionalFallbackIds.includes(id)?expectedBorrowedOptionalFallbackObservation(id,n,seed,inputText): dataUnionsIds.includes(id)?expectedDataUnionsObservation(id,n,seed,inputText):objectKeysIds.includes(id)?expectedObjectKeysObservation(id,n,seed,inputText):arrayFromIds.includes(id)?expectedArrayFromObservation(id,n,seed,inputText):stringAtIds.includes(id)?expectedStringAtObservation(id,n,seed,inputText):numberParseIntIds.includes(id)?expectedNumberParseIntObservation(id,n,seed):numberToStringIds.includes(id)?expectedNumberToStringObservation(id,n,seed):objectIsIds.includes(id)?expectedObjectIsObservation(id,n,seed,inputText):mathF16roundIds.includes(id)?expectedMathF16roundObservation(id,n,seed):mathRandomIds.includes(id)?expectedMathRandomObservation(id,n,seed):stringFromCodesIds.includes(id)?expectedStringFromCodesObservation(id,n,seed):stringCodeAtIds.includes(id)?expectedStringCodeAtObservation(id,n,seed):arrayEntriesIds.includes(id)?expectedArrayEntriesObservation(id,n,seed,inputText):tupleTypesIds.includes(id)?expectedTupleTypesObservation(id,n,seed,inputText):stringNormalizeIds.includes(id)?expectedStringNormalizeObservation(id,n,seed):arrayLocaleStringIds.includes(id)?expectedArrayLocaleStringObservation(id,n,seed,inputText):numberLocaleStringIds.includes(id)?expectedNumberLocaleStringObservation(id,n,seed):stringRawIds.includes(id)?expectedStringRawObservation(id,n,seed,inputText):stringLocaleCaseIds.includes(id)?expectedStringLocaleCaseObservation(id,n,seed):arrayKeysValuesIds.includes(id)?expectedArrayKeysValuesObservation(id,n,seed,inputText):arrayLiteralSpreadIds.includes(id)?expectedArrayLiteralSpreadObservation(id,n,seed,inputText):id==='logical_nullable_string_values'?expectedLogicalValueObservation(id,n,seed):callSpreadIds.includes(id)?expectedCallSpreadObservation(id,n,seed,inputText):restParameterIds.includes(id)?expectedRestParameterObservation(id,n,seed,inputText):optionalDefaultParameterIds.includes(id)?expectedOptionalDefaultParameterObservation(id,n,seed,inputText): nullableCallableIds.includes(id)?expectedNullableCallableObservation(id,n,seed,inputText): /^optional_chain_(?:int_field|number_field|boolean_field|string_length|array_index|array_join|record_field|nullable_chain)$/.test(id)?expectedOptionalChainObservation(id,n,seed,inputText): /^nullish_assign_(?:int|number|boolean|string|array|record|nullable_int|nullable_string)_value$/.test(id)?expectedNullishAssignmentObservation(id,n,seed,inputText): /^nullish_(?:(?:int|number|boolean|string|array|record|null_undefined)_value|nullable_fallback_int)$/.test(id)?expectedNullishObservation(id,n,seed,inputText): /^array_of_(?:int|number|boolean|string|nested|record|null|undefined)$/.test(id)?expectedOfObservation(id,n,seed,inputText): /^array_(?:flat|flat_map)_(?:int|number|boolean|string|nested|record|null|undefined)$/.test(id)?expectedFlatObservation(id,n,seed,inputText): /^array_(?:sort|to_sorted)_(?:default|compare)_(?:int|number|boolean|string|nested|record|null|undefined)$/.test(id)?expectedSortingObservation(id,n,seed,inputText): /^array_splice_(?:int|number|boolean|string|nested|record|null|undefined)$/.test(id)?expectedSpliceObservation(id,n,seed,inputText): /^array_unshift_(?:int|number|boolean|string|nested|record|null|undefined)$/.test(id)?expectedUnshiftObservation(id,n,seed,inputText): /^array_find_(?:last_)?(?:int|number|boolean|string|nested|record|null|undefined)$/.test(id)?expectedFindObservation(id,n,seed,inputText): arrayAtIds.includes(id)?expectedArrayAtObservation(id,n,seed,inputText): arrayRemovalIds.includes(id)?expectedArrayRemovalObservation(id,n,seed,inputText): narrowingIds.includes(id)?expectedNarrowingObservation(id,n,seed,inputText): nullableTransportIds.includes(id)?expectedNullableTransportObservation(id,n,seed,inputText): arrayToStringIds.includes(id)?expectedArrayToStringObservation(id,n,seed,inputText): arrayJoinIds.includes(id)?expectedArrayJoinObservation(id,n,seed,inputText): valueTypesIds.includes(id)?expectedValueTypesObservation(id,n,seed,inputText): htmlStringIds.includes(id)?expectedHtmlStringObservation(id,n,seed,inputText): stringSubstrIds.includes(id)?expectedStringSubstrObservation(id,n,seed,inputText): splitOwnershipIds.includes(id)?expectedSplitOwnershipObservation(id,n,seed,inputText): id.startsWith('template_')?expectedTemplateStringObservation(id,n,seed,inputText): id.startsWith('quoted_string_literals_')?expectedQuotedStringLiteralObservation(id,n,seed,inputText): /^string_(is|to)_well_formed_/.test(id)?expectedStringWellFormedObservation(id,n,seed,inputText): id.startsWith('number_parse_float_')?expectedNumberParseFloatObservation(id,n,seed,inputText): /^number_to_(fixed|exponential|precision)_/.test(id)?expectedNumberFormatObservation(id,n,seed,inputText): id.startsWith('int_to_string_')?expectedIntToStringObservation(id,n,seed,inputText): /^(string_(to_string|value_of)_(ascii|unicode)|boolean_(to_string|value_of)|number_value_of|int_value_of)$/.test(id)?expectedPrimitiveIdentityObservation(id,n,seed,inputText): id.startsWith('string_substring_')?expectedStringSubstringObservation(id,n,seed,inputText): /^string_(concat_(zero|variadic)_|pad_(start|end)_)/.test(id)?expectedStringConcatPaddingObservation(id,n,seed,inputText): id.startsWith('conditional_')?expectedConditionalObservation(id,n,seed,inputText): id.startsWith('compound_value_')?expectedCompoundValueObservation(id,n,seed): /^(int|number|boolean)_assignment_value$/.test(id)?expectedAssignmentValueObservation(id,n,seed): id.startsWith('boolean_logical_')?expectedLogicalAssignmentObservation(id,n,seed): id.startsWith('int_native_')?expectedNativeIntObservation(id,n,seed): id.startsWith('update_')?expectedUpdateObservation(id,n,seed): id.startsWith('compound_')?expectedCompoundObservation(id,n,seed,inputText): /^string_add_(direct|chained)_/.test(id)?expectedAddObservation(id,n,seed,inputText): /^string_trim_(start|end|left|right)_/.test(id)?expectedTrimObservation(id,n,seed,inputText): id.startsWith('string_last_index_of_')?expectedStringLastIndexObservation(id,n,seed,inputText): id.startsWith('array_last_index_of_')?expectedLastIndexObservation(id,n,seed,inputText): /^array_to_spliced_/.test(id)?expectedToSplicedObservation(id,n,seed,inputText): /^array_(to_reversed|with)_/.test(id)?expectedCopyingObservation(id,n,seed,inputText): /^array_(fill|copy_within)_/.test(id)?expectedRangeObservation(id,n,seed,inputText): id.startsWith('higher_')?expectedHigherObservation(id,n,seed,inputText): id.startsWith('declared_')?expectedDeclaredObservation(id,n,seed,inputText): id.startsWith('array_concat_') ? expectedConcatObservation(id,n,seed,inputText) : /^array_reduce(?:_right)?_/.test(id) ? expectedReductionObservation(id,n,seed) : /^string_(repeat_|replace_(first|all)_|split_)/.test(id) ? expectedConstructionObservation(id,n,seed,inputText) : /^array_(some|every|find_index|find_last_index)_/.test(id) ? expectedArrayPredicateObservation(id,n,seed,inputText) : id.startsWith('array_map_')||id.startsWith('array_filter_') ? expectedMapFilterObservation(id,n,seed,inputText) : id.startsWith('array_temporary_index_') ? expectedTemporaryObservation(id,n,seed,inputText) : id.startsWith('array_slice_') ? expectedSliceObservation(id,n,seed,inputText) : expectedNumberResult(id,n,seed); }
