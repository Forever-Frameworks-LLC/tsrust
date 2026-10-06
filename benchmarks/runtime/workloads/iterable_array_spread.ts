type ActionRow={label:string;action:()=>number;};
function workload(n: number, seed: number, input: string): string[][] {
 const output:string[][]=[];let i=0;
 while(i<n){
  const keys=new Set<number>([i+seed+2,i+seed,i+seed+2]);
  const copied=[0,...keys,i+seed+3];keys.clear();
  const characters=[...input,..."é🙂"];
  const values=new Map<string,number>([[String(input),i+seed],["tail",i+seed+1]]);
  const entries=[...values];const labels:string[]=[];
  for(const [key,value] of entries){labels.push(key+":"+String(value));}
  const rows=new Map<number,ActionRow>();rows.set(i+seed,{label:String(input),action:():number=>{return 7;}});
  const moved=[...rows];
  output.push([copied.join(","),characters.join("|"),String(entries.length),labels.join("|"),moved[0][1].label+":"+String(moved[0][0])]);i=i+1;
 }
 return output;
}
function main(){const result=workload(3,0,"abc");}
