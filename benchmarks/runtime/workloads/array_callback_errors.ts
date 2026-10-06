function workload(n: number, seed: number, input: string): string[][] {
 const output:string[][]=[];let i=0;
 while(i<n){
  const values=[i+seed+2,i+seed,i+seed+1];
  let caught="";let finalized=0;
  try{values.map((value:number):number=>{if(value===i+seed){throw String(input);}return value;});}
  catch(error){if(typeof error==="string"){caught=error;}}
  finally{finalized=1;}
  const sorted=values.toSorted((a:number,b:number):number=>{if(a<0){throw "negative";}return a-b;});
  output.push([caught,String(finalized),values.join(","),sorted.join(",")]);
  i=i+1;
 }
 return output;
}
function main(){const result=workload(3,0,"abc");}
