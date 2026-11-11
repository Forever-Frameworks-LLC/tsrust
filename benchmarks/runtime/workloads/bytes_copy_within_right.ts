/// <reference types="node" />
import {Buffer} from "node:buffer";
export function workload(n:number,seed:number,input:string):number[]{
  let size = 32;
  if (input === "4k") {size = 4096;}
  if (input === "1m") {size = 1048576;}
  const bytes=Buffer.alloc(size,97);
const output:number[]=[];for(let i=0;i<n;i++){bytes.copyWithin(1,0,size-1);output.push(bytes.length,bytes[0],bytes[bytes.length-1]);}return output;
}
