/// <reference types="node" />
import {Buffer} from "node:buffer";
export function workload(n:number,seed:number,input:string):number[]{
  let size = 32;
  if (input === "4k") {size = 4096;}
  if (input === "1m") {size = 1048576;}
  const bytes=Buffer.alloc(size,97);
  bytes[size-1]=98;
  const needle=Buffer.from([97,97,97,98]);
const output:number[]=[];for(let i=0;i<n;i++){output.push(bytes.indexOf(needle)+1,size,needle.length);}return output;
}
