import {observeSliceTree} from './slice-observation.mjs';

export function expectedGenericAdapterValues(id,n,seed,input) {
    if(id!=='generic_value_adapters'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||input!=='adapters')throw Error('Generic adapter case');
    return Array.from({length:n},(_,i)=>{
        const value=i%5+seed%5+1;
        return [value,value+1,7,value+2,5,value+3,value,value+1,value+11,value+12,1,value+13,value+2,value+21,5,value+31,8,value+3,6,value+4,6,1,0,value+2,value+2,1,value,1,9,2,3,5,value,1,4,4,5,value+5,value+41,value+6,value+42,value+7,value+8,value+51,value+52,value+9,value+61,value+11,value+12,value+71,value+72,value+13,value+82,value+84,value+15,value+16,value+91,value+92,value+17,value+101,value+102,value+19,value+20,value+113,value+115,2,2,value+21,value+121,value+122,value+23,value+24,value+133,value+135,4,2,value+25,value+28,value+29,1,value+31,value+161,5,value+171,value+33,value+181,value+34,value+182,value+35,value+191,1,value+37,value+38,value+40,2,1,value+41,value+211,5,value+212,value+213,value+42,value+231,1,0,0,value+43,value+261,5,value+262,value+44,value+271,4,7,14,1,value+45,value+301,5,value+311,value+46,7,value+47,8,value+48,4,value+49,value+50+(value%2===0?1:2),value%2===0?value*2+52:(value+50)*(value+2),value+(value%2===0?0:1),value*4+(value%2===0?15:40),value+(value%2===0?62:72)];
    });
}

export function expectedGenericAdapterObservation(id,n,seed,input) {
    return observeSliceTree(expectedGenericAdapterValues(id,n,seed,input));
}
