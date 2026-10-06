import {observeSliceTree} from './slice-observation.mjs';
export function expectedScalarUnaryPlusValues(id,n,seed,input){
 if(id!=='scalar_unary_plus'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||input!=='invalid')throw Error('Unary plus case');
 return Array.from({length:n},(_,i)=>{const k=i+seed;const decimal=String(k)+'.5';const converted=k%2===0?decimal:'NaN';return [decimal,k%2===0?'1':'0','0','NaN',converted,converted,'true'];});
}
export function expectedScalarUnaryPlusObservation(id,n,seed,input){return observeSliceTree(expectedScalarUnaryPlusValues(id,n,seed,input));}
