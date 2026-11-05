/// <reference types="node" />
import {Buffer} from "node:buffer";
export function workload(n:number,seed:number,input:string):number[]{
  let size = 64;
  if (input === "4k") {size = 4096;}
  if (input === "1m") {size = 1048576;}
  const bytes=Buffer.alloc(size);const records=size/64;
const output:number[]=[];for(let i=0;i<n;i++){for(let record=0;record<records;record++){const base=record*64;const suffix=record%2===0?"LE":"BE";bytes[`writeInt${suffix}`](65,base+1,1);bytes[`writeInt${suffix}`](-12345,base+3,2);bytes[`writeInt${suffix}`](-1234567,base+5,4);bytes[`writeBigInt64${suffix}`](123456789n,base+9);bytes[`writeUInt${suffix}`](255,base+17,1);bytes[`writeUInt${suffix}`](65535,base+19,2);bytes[`writeUInt${suffix}`](4294967295,base+21,4);bytes[`writeFloat${suffix}`](1.5,base+25);bytes[`writeDouble${suffix}`](-2.25,base+33);}let checksum=0;for(const byte of bytes){checksum+=byte;}output.push(size,checksum,records);}return output;
}
