import http from 'node:http';
import {Readable} from 'node:stream';
function size(id:number):number{return id%3===1?64:id%3===2?4096:65537;}
function payload(count:number,offset:number,id:number):Buffer{const bytes=Buffer.alloc(count);for(let i=0;i<count;i++)bytes[i]=((offset+i)*17+id)%251;return bytes;}
async function main():Promise<void>{
 const backend=http.createServer((request,response)=>{const id=Number(request.url!.split('/')[3]);let offset=0,checksum=0;request.on('data',(chunk:Buffer)=>{for(const byte of chunk){if(byte!==(offset*17+id)%251)throw Error('upload bytes/order mismatch');offset++;checksum=(checksum*131+byte)%1000000007;}});request.on('end',()=>{if(offset!==size(id))throw Error('upload length mismatch');response.setHeader('content-type','text/plain; charset=utf-8');response.end(`${offset},${checksum}`);});});
 await new Promise<void>(resolve=>backend.listen(0,'127.0.0.1',resolve));const port=(backend.address()as import('node:net').AddressInfo).port;const agent=new http.Agent({keepAlive:true});
 const server=http.createServer((input,response)=>{const[,mode,idText]=input.url!.split('/');const id=Number(idText),count=size(id);const request=http.request({host:'127.0.0.1',port,path:`/sink/${mode}/${id}`,method:'POST',agent,headers:mode==='bytes'?{'content-length':count}:{}},upstream=>{response.setHeader('content-type','text/plain; charset=utf-8');upstream.pipe(response);});request.on('error',error=>response.destroy(error));
  if(mode==='bytes')request.end(payload(count,0,id));
  else if(mode==='stream')Readable.from([payload(count,0,id)]).pipe(request);
  else{const first=Math.floor(count/2);Readable.from((async function*(){yield payload(first,0,id);yield payload(count-first,first,id);})(),{objectMode:false,highWaterMark:1}).pipe(request);}
 });await new Promise<void>(resolve=>server.listen(0,'127.0.0.1',resolve));console.log(JSON.stringify({address:`127.0.0.1:${(server.address()as import('node:net').AddressInfo).port}`}));
 await new Promise<void>(resolve=>process.once('SIGINT',resolve));await new Promise<void>(resolve=>server.close(()=>resolve()));agent.destroy();await new Promise<void>(resolve=>backend.close(()=>resolve()));
}
void main().catch(error=>{console.error(error);process.exitCode=1;});
