type ActionRow={label:string;action:()=>number;};
function workload(n: number, seed: number, input: string): string[][] {
 const output:string[][]=[];let i=0;
 while(i<n){
  const keys=new Set<number>([i+seed+2,i+seed,i+seed+2]);
  const copied=Array.from(keys);keys.clear();
  const mapped=Array.from(new Set<number>([i+seed+2,i+seed]),(value,index)=>{return value*10+index;});
  const values=new Map<string,number>([[String(input),i+seed],["tail",i+seed+1]]);
  const entries=Array.from(values);
  const labels=Array.from(values,([key,value],index)=>{return key+":"+String(value+index);});
  const rows=new Map<number,ActionRow>();rows.set(i+seed,{label:String(input),action:():number=>{return 7;}});
  const read=Array.from(rows,(entry,index)=>{return entry[1].label+":"+String(entry[0]+index);});
  output.push([copied.join(","),mapped.join(","),String(entries.length),labels.join("|"),read.join("|")]);i=i+1;
 }
 return output;
}
function main(){const result=workload(3,0,"abc");}
