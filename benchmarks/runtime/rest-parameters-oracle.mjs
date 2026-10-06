import {ids,expectedValues} from './rest-parameters-values-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
export {ids as restParameterIds};
export function expectedRestParameterObservation(id,n,seed,input){return observeSliceTree(expectedValues(id,n,seed,input),{numbers:id==='rest_parameter_number_pack'});}
