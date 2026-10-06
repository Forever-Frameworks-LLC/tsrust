import {observeSliceTree} from './slice-observation.mjs';
export const bracelessBodiesIds=['braceless_bodies'];
// Independent of the loops: the counted sum is a triangular number, the grid
// count is 3 diagonal cells plus 2 off-diagonal anti-diagonal cells times 10,
// and the counter always settles at 1 and then steps by 2 to 5.
export function expectedBracelessBodiesValues(id,n,seed,input){
 if(!bracelessBodiesIds.includes(id)||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Braceless body case');
 const result=[];for(let i=0;i<n;i++){const base=(i*13+seed)%97,count=base%5+1;result.push([base<20?'low':base<60?'mid':'high',''+(count*(count-1)/2),input+base,'23','5']);}return result;
}
export function expectedBracelessBodiesObservation(id,n,seed,input){return observeSliceTree(expectedBracelessBodiesValues(id,n,seed,input));}
