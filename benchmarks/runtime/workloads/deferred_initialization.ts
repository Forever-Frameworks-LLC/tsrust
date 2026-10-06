type int=number;
function workload(n: int, seed: int, input: string): string[][] {
 const output:string[][]=[];let i=0;
 while (i < n) {
  const base=(i*13+seed)%97;
  let label:string;
  if(base>=64){label="high";}else if(base>=32){label="mid";}else{label="low";}
  let size:int,step:int=base%5;
  size=step*2+1;
  let kind:string;
  switch(step){case 0:kind="zero";break;case 1:case 2:kind="few";break;default:kind="many";}
  let text:string;
  {const prefix=String(base);text=prefix+input;}
  let note:string|undefined;
  const absent=note===undefined;
  output.push([label,String(size),kind,text,String(absent)]);
  i=i+1;
 }return output;
}
function main(){const result=workload(3,0,"abc");}
