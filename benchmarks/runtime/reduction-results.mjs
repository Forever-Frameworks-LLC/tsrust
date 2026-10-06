// The expected workload values come from independent traversal/BigInt quarters.
// This external observation dispatch does not call the timed reduction API.
import {expectedReduction} from './reduction-oracle.mjs';
import {observeNumber} from './number-observation.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {observeReduction} from './reduction-observation.mjs';
const types={int:'int',number:'number',boolean:'boolean',string:'string',record:'Row',nested_int:'int[][]',nested_number:'number[][]',nested_boolean:'boolean[][]',nested_string:'string[][]',nested_record:'Row[][]',deep_int:'int[][][]',int_to_record:'Row',int_to_array:'int[][]',int_to_nested:'int[][][]'};
export function expectedReductionObservation(id,n,seed){
 const match=/^array_reduce(?:_right)?_(.+)$/.exec(id),type=types[match?.[1]];
 if(!type)throw Error('Unsupported reduction result schema');const value=expectedReduction(id,n,seed);
 if(type==='int')return {checksum:value};
 if(type==='number')return observeNumber(value);
 if(['boolean','string','Row','boolean[][]','Row[][]'].includes(type))return observeReduction(value,type);
 return observeSliceTree(value,{numbers:type==='number[][]'});
}
