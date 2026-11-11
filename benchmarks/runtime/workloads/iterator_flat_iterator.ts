function* source(values:number[]):Generator<number>{yield* values;}
function* filter(values:Iterable<number>,callback:(value:number)=>boolean):Generator<number>{for(const value of values)if(callback(value))yield value;}
function* map(values:Iterable<number>,callback:(value:number)=>number):Generator<number>{for(const value of values)yield callback(value);}
function* take(values:Iterable<number>,count:number):Generator<number>{if(count===0)return;for(const value of values){yield value;if(--count===0)return;}}
function* flat(values:Iterable<number>):Generator<number>{for(const value of values)yield* source([value,value+1]);}
function fold(values:Iterable<number>):number{let sum=7;for(const value of values)sum=(sum*131+value)%1000000007;return sum;}
function workload(n:number,seed:number):number{let result=0;for(let i=0;i<n;i++){const values:number[]=[];for(let j=0;j<64;j++)values.push((j*17+seed+i)%997);const observed=fold(flat(source(values)));result=(result*131+observed)%1000000007;}return result;}
