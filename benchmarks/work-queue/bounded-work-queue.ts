import {performance} from 'node:perf_hooks';
import assert from 'node:assert/strict';
type Work = {id:number; payload:string; stop:boolean};
class Queue<T> {
  private readonly items:T[]=[];
  private readonly readers:{resolve:(value:T)=>void; reject:(error:Error)=>void}[]=[];
  private readonly writers:(()=>void)[]=[];
  private closed=false;
  constructor(private readonly capacity:number){}
  async send(value:T):Promise<void>{
    while(this.items.length===this.capacity&&!this.closed) await new Promise<void>(resolve=>this.writers.push(resolve));
    if(this.closed) throw new Error('closed');
    const reader=this.readers.shift();
    if(reader) reader.resolve(value);else this.items.push(value);
  }
  async receive():Promise<T>{
    if(this.items.length){const value=this.items.shift()!;this.writers.shift()?.();return value;}
    if(this.closed) throw new Error('closed');
    return new Promise<T>((resolve,reject)=>this.readers.push({resolve,reject}));
  }
  close():void{
    this.closed=true;
    for(const wake of this.writers.splice(0))wake();
    for(const reader of this.readers.splice(0))reader.reject(new Error('closed'));
  }
}
async function produce(queue:Queue<Work>):Promise<number>{
  for(let id=1;id<=64;id++)await queue.send({id,payload:'job-'+id.toString(),stop:false});
  for(let worker=0;worker<2;worker++)await queue.send({id:0,payload:'',stop:true});
  return 0;
}
async function consume(queue:Queue<Work>):Promise<number>{
  let total=0;
  while(true){const work=await queue.receive();if(work.stop)return total;assert.equal(work.payload,'job-'+work.id.toString());total+=work.id;await Promise.resolve();}
}
async function batch():Promise<number>{
  const queue=new Queue<Work>(2);
  const results=await Promise.allSettled([produce(queue),consume(queue),consume(queue)]);
  let total=0;for(const result of results){if(result.status==='rejected')throw result.reason;total+=result.value;}
  queue.close();return total;
}
async function main(){
  for(let warmup=0;warmup<8;warmup++)await batch();
  for(let sample=0;sample<31;sample++){
    const start=performance.now();const checksum=await batch();const elapsed=performance.now()-start;
    console.log(checksum);console.log(elapsed);
  }
}
void main();
