import {observeSliceTree} from './slice-observation.mjs';
export function expectedNumericStringConversionValues(id,n,seed,input){
 if(id!=='numeric_string_conversion'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||input!=='0x10')throw Error('Numeric text case');
 const values=['16','5','15','12','0','NaN','Infinity','0','9007199254740996','NaN'];
 return Array.from({length:n},(_,i)=>{const k=i+seed;const at=k%values.length;const text=values[at];return [text,text,k%2===0?text:'0',at===4||at===7?'true':'false'];});
}
export function expectedNumericStringConversionObservation(id,n,seed,input){return observeSliceTree(expectedNumericStringConversionValues(id,n,seed,input));}
