import {observeSliceTree} from './slice-observation.mjs';
// Independent of declared shapes: every cell is direct text and arithmetic.
export function expectedInterfacesValues(id,n,seed,input){
 if(id!=='interfaces'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Interface case');
 return Array.from({length:n},(_,i)=>{const k=i+seed;return [input+':'+k+':'+(k%5),'p:'+(k+k%3)+':1',''+k,''+(k%5+1),''+(k%3)];});
}
export function expectedInterfacesObservation(id,n,seed,input){return observeSliceTree(expectedInterfacesValues(id,n,seed,input));}
