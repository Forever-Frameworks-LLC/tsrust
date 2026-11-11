/// <reference types="node" />
import {Buffer} from "node:buffer";
export function workload(n:number,seed:number,input:string):number[]{
  let size = 32;
  if (input === "4k") {size = 4096;}
  if (input === "1m") {size = 1048576;}
  const bytes=Buffer.alloc(size,97);
  const other=Buffer.alloc(size,97);
const output:number[]=[];for(let i=0;i<n;i++){output.push(Number(bytes.equals(other)),size,other.length);}return output;
}
