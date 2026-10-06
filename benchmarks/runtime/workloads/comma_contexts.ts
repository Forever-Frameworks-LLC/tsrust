type Row={value:number|null;};
function mark(trace:number[],value:number):number{trace.push(value);return value;}
function reset(row:Row):number{row.value=null;return 0;}
function workload(n: number, seed: number, input: string): string[][]{
 const output:string[][]=[];
 for(let i=0;i<n;i++){
  const trace:number[]=[];const j=seed+i;const values=[j,j+1];
  (mark(trace,1),values).push(j+2);(mark(trace,2),values).fill(8,1,2);(mark(trace,3),values).reverse();
  const joined=(mark(trace,4),values).join(",");
  const value:number|null=i%2===0?j:null;let narrowed=0;
  if(value!==(mark(trace,5),null)){narrowed=value+1;}
  const alternative:number|string=i%2===0?"yes":j;let text="number";
  if((mark(trace,6),typeof alternative)===(mark(trace,7),"string")){text=(alternative as string).toUpperCase();}
  const row:Row={value:j};let changed=false;
  if(row.value!==(reset(row),null)){changed=row.value===null;}
  output.push([joined,trace.join(","),String(narrowed),text,String(changed),String(input)]);
 }
 return output;
}
function main(){const result=workload(3,0,"abc");}
