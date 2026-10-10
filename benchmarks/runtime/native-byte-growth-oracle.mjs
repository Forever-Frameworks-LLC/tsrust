import {observeSliceTree} from './slice-observation.mjs';
export function expectedNativeByteGrowthValues(id,n,seed,input) {
    if(id!=='native_byte_growth'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||input!=='bytes') throw Error('Native byte growth case');
    return Array.from({length:n},(_,i)=>{const byte=(i+seed)%256;return [byte,1,0,10,4,1,1,byte,255,255,3];});
}
export function expectedNativeByteGrowthObservation(id,n,seed,input) { return observeSliceTree(expectedNativeByteGrowthValues(id,n,seed,input)); }
