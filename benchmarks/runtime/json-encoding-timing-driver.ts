declare const process:{argv:string[]};
const n = Number(process.argv[2]);
const cold=performance.now();
let result:unknown=workload(0, 7);
const coldMs=performance.now()-cold;
const checksum=probeChecksum(result);
result=undefined;
const durations:number[]=[];
for(let batch=0;batch<39;batch++){
 const start=performance.now();
 for(let i=0;i<100;i++)result=workload(n, 7);
 const elapsed=performance.now()-start;
 if(probeChecksum(result)!==checksum)throw Error('Unstable output');
 result=undefined;
 if(batch>=8)durations.push(elapsed);
}
console.log(JSON.stringify({checksum,coldMs,durations,iterations:100,warmups:8}));
