import {observeSliceTree} from './slice-observation.mjs';
export const objectKeysIds=['object_keys_order'];
// Derive constructor names and UTF-16 positions without Object.keys or record copies.
export function expectedObjectKeysValues(id,n,seed,input){
 if(!objectKeysIds.includes(id)||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Object keys bounds');
 const textNames=[];for(let j=0;j<input.length;j++)textNames.push(String(j));
 const output=[];for(let i=0;i<n;i++){const even=(i+seed)%2===0,names=even?'gamma|alpha|beta':'beta|gamma|alpha';output.push([names,names,even?'changed|alpha|beta':'changed|gamma|alpha',textNames.join('|'),String((i*13+seed)%97)]);}return output;
}
export function expectedObjectKeysObservation(id,n,seed,input){return observeSliceTree(expectedObjectKeysValues(id,n,seed,input));}
