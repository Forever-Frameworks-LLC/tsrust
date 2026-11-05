/// <reference types="node" />
import {Buffer} from "node:buffer";
export function workload(n:number,seed:number,input:string):number[]{
  let size=256; if(input==="adversarial-medium"){size=4096;} if(input==="adversarial-large"){size=65536;}
  const bytes=Buffer.alloc(size,97);
  const needle=Buffer.alloc(Math.min(size,512)/2,97);needle[needle.length-1]=98;
const output:number[]=[];for(let i=0;i<n;i++){output.push(bytes.indexOf(needle)+1,size,needle.length);}return output;
}
