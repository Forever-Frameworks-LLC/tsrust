import {observeSliceTree} from './slice-observation.mjs';
export function expectedMapSnapshotsValues(id,n,seed,input){
 if(id!=='map_snapshots'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Map snapshot case');
 return Array.from({length:n},(_,i)=>[`${i+seed+1},${i+seed}`,`tail|${input}`,String(i+seed+1),'tail',String(i+seed),input,'0']);
}
export function expectedMapSnapshotsObservation(id,n,seed,input){return observeSliceTree(expectedMapSnapshotsValues(id,n,seed,input));}
