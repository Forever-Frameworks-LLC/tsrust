import {observeSliceTree} from './slice-observation.mjs';

export function expectedGenericAdapterValues(id,n,seed,input) {
    if(id!=='generic_value_adapters'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||input!=='adapters')throw Error('Generic adapter case');
    return Array.from({length:n},(_,i)=>{
        const value=i%5+seed%5+1;
        return [value,value+1,7,value+2,5,value+3,value,value+1,value+11,value+12,1,value+13,value+2,value+21,5,value+31,8,value+3,6,value+4,6,1,0,value+2,value+2,1];
    });
}

export function expectedGenericAdapterObservation(id,n,seed,input) {
    return observeSliceTree(expectedGenericAdapterValues(id,n,seed,input));
}
