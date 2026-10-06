import {ids as baseIds,expectedNullableCallableValues} from './nullable-callable-values-oracle.mjs';
import {ids as higherIds,expected as expectedHigher} from './nullable-callable-higher-order-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
export const nullableCallableIds=Object.freeze([...baseIds,...higherIds]);
export function expectedNullableCallableObservation(id,n,seed,input){return observeSliceTree(baseIds.includes(id)?expectedNullableCallableValues(id,n,seed,input):expectedHigher(id,n,seed,input));}
