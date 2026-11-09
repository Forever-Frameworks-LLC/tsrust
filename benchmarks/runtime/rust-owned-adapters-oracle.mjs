export const rustOwnedAdapterIds=['rust_owned_scalar','rust_owned_strings','rust_owned_records','rust_owned_nested_option'];
export function expectedRustOwnedAdapterChecksum(id,n,seed){
 if(!rustOwnedAdapterIds.includes(id))throw Error('Unknown owned adapter workload');
 let checksum=0;for(let i=0;i<n;i++){const value=seed+i;let observed;
 if(id==='rust_owned_scalar')observed=value*3+1;
 else if(id==='rust_owned_strings')observed=value+14;
 else if(id==='rust_owned_records')observed=value*2+10;
 else observed=value%3===0?1:value%3===1?2:12;
 checksum=(checksum*131+observed)%1000000007;
 }return checksum;
}
