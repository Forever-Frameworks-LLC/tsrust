declare const process:{argv:string[]};
const n = Number(process.argv[2]);
const call=()=>n<0?workload(0, 7):workload(n, 7);
const cold=performance.now();
let result:unknown=call();
const coldMs=performance.now()-cold;
const checksum=probeChecksum(result);
result=undefined;
const durations:number[]=[];
for(let batch=0;batch<39;batch++){
 const start=performance.now();
 for(let i=0;i<1;i++)result=call();
 const elapsed=performance.now()-start;
 if(probeChecksum(result)!==checksum)throw Error('Unstable output');
 result=undefined;
 if(batch>=8)durations.push(elapsed);
}
console.log(JSON.stringify({checksum,coldMs,durations,iterations:1,warmups:8}));
