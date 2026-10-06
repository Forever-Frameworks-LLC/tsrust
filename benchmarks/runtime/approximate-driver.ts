// New protocol only. Observation and retention occur outside the timed body.
const host=globalThis as unknown as {Deno?:{args:string[]};process?:{argv:string[]}};
const argv=host.Deno?host.Deno.args:host.process!.argv.slice(2);
const iterations=Number(argv[0]),seed=Number(argv[1]),warmups=Number(argv[2]),samples=Number(argv[3]);
const view=new DataView(new ArrayBuffer(8));
function actualScalarBits(value:number):string{view.setFloat64(0,value,false);return view.getBigUint64(0,false).toString(16).padStart(16,'0');}
const warmupBits:string[]=[],actualBits:string[]=[],durations:number[]=[];
for(let i=0;i<warmups;i++)warmupBits.push(actualScalarBits(workload(iterations,seed)));
for(let i=0;i<samples;i++){const start=performance.now(),value=workload(iterations,seed);durations.push(performance.now()-start);actualBits.push(actualScalarBits(value));}
console.log(JSON.stringify({protocol:'mpfr-directed-f64-scalar-v1',warmupBits,actualBits,durations}));
