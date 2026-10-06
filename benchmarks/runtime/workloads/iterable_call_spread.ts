// TypeScript requires rest signatures for dynamic iterable spread; each scenario supplies the native checked count.
type ActionRow={label:string;action:()=>number;};
function total(...values:number[]):number{let result=0;for(const value of values){result+=value;}return result;}
function letters(...values:string[]):string{return values.join("|");}
function pair(...entries:[number,number][]):number{const a=entries[0];const b=entries[1];return a[0]+a[1]+b[0]+b[1];}
function first<T>(...values:T[]):T{return values[0];}
function label(...entries:[number,ActionRow][]):string{const entry=entries[0];return entry[1].label+":"+String(entry[0]);}
function workload(n: number, seed: number, input: string): string[][] {
 const output:string[][]=[];let i=0;
 while(i<n){
  const keys=new Set<number>([i+seed+2,i+seed,i+seed+2]);
  const sum=total(...keys);keys.clear();
  const characters=letters(...input);
  const values=new Map<number,number>([[i+seed,2],[i+seed+1,3]]);
  const combined=pair(...values);
  const selected=first(...new Set<number>([i+seed+2]));
  const rows=new Map<number,ActionRow>();rows.set(i+seed,{label:String(input),action:():number=>{return 7;}});
  const text=label(...rows);
  const raw=String.raw({raw:["<",":",">"]},...new Set<string>(["a","b"]));
  output.push([String(sum),characters,String(combined),String(selected),text,String(values.size),raw]);i=i+1;
 }
 return output;
}
function main(){const result=workload(3,0,"abc");}
