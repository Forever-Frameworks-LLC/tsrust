/// <reference types="node" />
import {deepStrictEqual,notDeepStrictEqual} from 'node:assert/strict';
type int=number;
function workload(n:int,seed:int):int{
 const first=new Map<int,int[]>();const second=new Map<int,int[]>();const different=new Map<int,int[]>();const setA=new Set<int>();const setB=new Set<int>();let j=0;
 while(j<8){first.set(j,[j*j,j+1]);second.set(7-j,[(7-j)*(7-j),8-j]);different.set(j,[j*j,j+2]);setA.add(j);setB.add(7-j);j++;}
 let checksum=0;let i=0;while(i<n){const at=(i*13+seed)%8;deepStrictEqual(first,second);deepStrictEqual(setA,setB);notDeepStrictEqual(first,different);const found=first.get(at);const actual=found===undefined?-1:found[0];checksum=(checksum*131+actual)%1000000007;i++;}return checksum;
}
function main(){console.log(workload(17,3));}
