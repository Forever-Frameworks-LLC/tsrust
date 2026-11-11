/// <reference types="node" />
import {deepStrictEqual,notDeepStrictEqual} from 'node:assert/strict';
type int=number;
type Row={key:int;label:string;values:number[];};
function workload(n:int,seed:int):int{
 const left:Row[]=[];const right:Row[]=[];let j=0;
 while(j<8){left.push({key:j*j,label:'é',values:[Number.NaN,-0.0,Number(j)]});right.push({key:j*j,label:'é',values:[Number.NaN,-0.0,Number(j)]});j++;}
 let checksum=0;let i=0;while(i<n){const at=(i*13+seed)%8;deepStrictEqual(left[at],right[at]);notDeepStrictEqual(left[at],right[(at+1)%8]);checksum=(checksum*131+left[at].key)%1000000007;i++;}return checksum;
}
function main(){console.log(workload(17,3));}
