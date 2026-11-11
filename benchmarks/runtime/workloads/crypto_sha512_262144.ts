import {createHash,createHmac,hkdfSync,timingSafeEqual} from 'node:crypto';
export function workload(n:number,seed:number):number{const size=262144;const input=inputBytes(size,seed);let sum=0;for(let i=0;i<n;i++){const output=createHash('sha512').update(input).digest();sum=fold(sum,output);}return sum;}
function inputBytes(size:number,seed:number):Buffer{const bytes=Buffer.alloc(size);for(let i=0;i<size;i++)bytes[i]=(i+seed)%251;return bytes;}
function fold(sum:number,bytes:Uint8Array):number{for(const byte of bytes)sum=(sum*131+byte)%1000000007;return sum;}
