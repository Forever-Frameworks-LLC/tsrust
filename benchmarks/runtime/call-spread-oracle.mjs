import {ids,expectedValues} from './call-spread-values-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
export {ids as callSpreadIds};
export function expectedCallSpreadObservation(id,n,seed,input){return observeSliceTree(expectedValues(id,n,seed,input),{numbers:id==='call_spread_math_number'});}
