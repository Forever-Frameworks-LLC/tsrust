// Workload setup/drain is observed per invocation; no reporting occurs in timed code.
async function benchmark_main():Promise<void>{
 const host=globalThis as unknown as {Deno?:{args:string[]};process?:{argv:string[]}};
 const argv=host.Deno?host.Deno.args:host.process!.argv.slice(2);
 const iterations=Number(argv[0]),seed=Number(argv[1]),warmups=Number(argv[2]),samples=Number(argv[3]);
 let expected:number|undefined;function validate(value:number):void{if(!Number.isSafeInteger(value))throw new Error('Checksum outside safe integer domain');if(expected!==undefined&&value!==expected)throw new Error('Unstable checksum');expected=value;}
 for(let i=0;i<warmups;i++)validate(await workload(iterations,seed));
 const durations:number[]=[];for(let i=0;i<samples;i++){const start=performance.now();const value=await workload(iterations,seed);durations.push(performance.now()-start);validate(value);}
 console.log(JSON.stringify({checksum:expected,durations}));
}
void benchmark_main().catch(error=>{console.error(error);throw error;});
