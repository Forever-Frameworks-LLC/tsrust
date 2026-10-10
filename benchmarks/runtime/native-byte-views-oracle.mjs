import {observeSliceTree} from './slice-observation.mjs';
export function expectedNativeByteViewsValues(id, n, seed, input) {
    if (id !== 'native_byte_views' || !Number.isSafeInteger(n) || n < 0 || n > 2048 || !Number.isSafeInteger(seed) || seed < 0 || seed > 1000000 || input !== 'bytes') throw Error('Native byte views case');
    return Array.from({length:n}, (_,i) => { const byte=(i+seed)%256; return [3,byte,255,(byte+1)%256,2,1,11,2,(byte+2)%256,5,9,1]; });
}
export function expectedNativeByteViewsObservation(id,n,seed,input) { return observeSliceTree(expectedNativeByteViewsValues(id,n,seed,input)); }
