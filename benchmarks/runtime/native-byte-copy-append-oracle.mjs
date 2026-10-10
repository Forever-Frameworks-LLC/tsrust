import {observeSliceTree} from './slice-observation.mjs';
export function expectedNativeByteCopyAppendValues(id,n,seed,input) {
    if(id!=='native_byte_copy_append'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||input!=='bytes')throw Error('Native byte copy/append case');
    return Array.from({length:n},(_,i)=>[2,1,10,1,7,2,1,(i+seed)%256,1,65,0,5,4]);
}
export function expectedNativeByteCopyAppendObservation(id,n,seed,input){return observeSliceTree(expectedNativeByteCopyAppendValues(id,n,seed,input));}
