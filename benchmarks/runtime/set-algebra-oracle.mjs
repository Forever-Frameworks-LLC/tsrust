import {observeSliceTree} from './slice-observation.mjs';
export function expectedSetAlgebraValues(id,n,seed,input){
 if(id!=='set_algebra'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Set algebra case');
 const labels=[input,'tail','last'].filter((value,index,all)=>all.indexOf(value)===index).map(value=>value+'|').join('');
 return Array.from({length:n},(_,i)=>{const k=i+seed;return [[k+3,k+1,k+2,k+4].join(','),[k+1,k+2].join(','),String(k+3),[k+3,k+4].join(','),[k+2,k+1].join(','),labels];});
}
export function expectedSetAlgebraObservation(id,n,seed,input){return observeSliceTree(expectedSetAlgebraValues(id,n,seed,input));}
