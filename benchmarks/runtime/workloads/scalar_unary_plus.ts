type int = number;
function workload(n: int, seed: int, input: string): string[][] {
  const output:string[][]=[];let i=0;
  while(i<n){
    const k=i+seed;const flag=k%2===0;const text=String(k)+".5";
    const nullable:string|undefined=flag?String(text):undefined;
    output.push([String(+text),String(+flag),String(+(null as any)),String(+(undefined as any)),String(+(nullable as any)),String(Number(nullable)),String(Number.isNaN(+input))]);i+=1;
  }
  return output;
}
function main(){const result=workload(3,0,"invalid");}
