import {ids,expectedValues} from './optional-default-parameters-values-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
export {ids as optionalDefaultParameterIds};
export function expectedOptionalDefaultParameterObservation(id,n,seed,input){return observeSliceTree(expectedValues(id,n,seed,input),{numbers:id==='default_parameter_number_value'});}
