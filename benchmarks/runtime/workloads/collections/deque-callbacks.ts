type Reader = { read: () => number; };
function byteWindow(owner:Uint8Array):Uint8Array{return owner.subarray(0,2);}
function first(part:Uint8Array):number{return part[0];}
function workload(n:number,seed:number,input:string):number[][]{
 const output:number[][]=[];let i=0;while(i<n){const byte=(i+seed)%256;const owner=Uint8Array.from([byte,255]);
 const captured=byteWindow(owner);const values:Reader[]=[];let count=0;while(count<8){values.push({read:()=>first(captured)});count++;}
 values.unshift({read:()=>first(captured)});let read=0;{const selected=values[4];if(selected!==undefined){read=selected.read();}}
 let traversed = 0; for (const part of values) { traversed += part.read(); }
const front=values.shift()!;const back=values.pop()!;let sum=front.read()+back.read();
 while(values.length>0){const part=values.shift()!;sum+=part.read();}
 values.push({read:()=>first(captured)});values.length=0;output.push([count+1,read,sum,values.length,traversed]);
 i++;}return output;
}
function main(){}
