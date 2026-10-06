function workload(n: number, seed: number, input: string): string[][] {
 const output:string[][]=[];let i=0;
 while(i<n){
  const values=new Map<string,number>([[String(input),i+seed],["tail",i+seed+1]]);
  let direct="";for(const entry of values){direct=direct+entry[0]+":"+String(entry[1])+"|";}
  let pattern="";for(const [key,value] of values){pattern=pattern+key+":"+String(value)+"|";}
  values.set("last",i+seed+2);let count=0;for(const entry of values){count+=1;}
  output.push([direct,pattern,String(count)]);i=i+1;
 }
 return output;
}
function main(){const result=workload(3,0,"abc");}
