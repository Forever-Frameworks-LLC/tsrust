import {observeSliceTree} from './slice-observation.mjs';
export function expectedNativeByteViewsValues(id, n, seed, input) {
    if (!['native_byte_views','native_byte_views_maps','native_byte_views_recursive','native_byte_views_linked','native_byte_views_callbacks','native_byte_views_stored_callbacks','native_byte_views_captured_only','native_byte_views_nullable_callbacks','native_byte_views_parameter_stores'].includes(id) || !Number.isSafeInteger(n) || n < 0 || n > 2048 || !Number.isSafeInteger(seed) || seed < 0 || seed > 1000000 || input !== 'bytes') throw Error('Native byte views case');
    if (id === 'native_byte_views_maps') return Array.from({length:n},(_,i)=>[8,28,8*((i+seed)%256),(i+seed)%256,2]);
    if (id === 'native_byte_views_parameter_stores') return Array.from({length:n},(_,i)=>[17,16*((i+seed+1)%256),1]);
    if (id === 'native_byte_views_nullable_callbacks') return Array.from({length:n},(_,i)=>[17,16*((i+seed)%256),1]);
    if (id === 'native_byte_views_callbacks') return Array.from({length:n},(_,i)=>[2,(i+seed+1)%256,2,(i+seed+1)%256]);
    if (['native_byte_views_maps','native_byte_views_recursive','native_byte_views_linked','native_byte_views_stored_callbacks','native_byte_views_captured_only'].includes(id)) return Array.from({length:n}, (_,i) => [17,17*((i+seed)%256),2]);
    return Array.from({length:n}, (_,i) => { const byte=(i+seed)%256; return [3,byte,255,(byte+1)%256,2,1,11,2,(byte+2)%256,5,9,1]; });
}
export function expectedNativeByteViewsObservation(id,n,seed,input) { return observeSliceTree(expectedNativeByteViewsValues(id,n,seed,input)); }
