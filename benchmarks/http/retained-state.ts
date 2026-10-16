declare function require(name: string): any;
const http = require('node:http');
async function batch(url: string): Promise<number> {
  let sum = 0;
  for (let index = 0; index < 128; index++) {
    const response = await fetch(url, { redirect: 'manual' });
    if (response.status !== 200 || await response.text() !== '42') throw new Error('Unexpected response');
    sum += 42;
  }
  return sum;
}
async function main(): Promise<void> {
  const config: Readonly<{body:string}> = {body:'42'};
  const server = http.createServer((_request: any, response: any) => {response.setHeader('content-type', 'text/plain; charset=utf-8');response.end(config.body);});
  await new Promise<void>(resolve => server.listen(0, '127.0.0.1', resolve));
  const url = `http://127.0.0.1:${server.address().port}/`;
  try {
    for (let warmup = 0; warmup < 8; warmup++) await batch(url);
    for (let sample = 0; sample < 31; sample++) {const start = performance.now();const sum = await batch(url);console.log(sum);console.log(performance.now() - start);}
  } finally {await new Promise<void>(resolve => {server.close(resolve);server.closeIdleConnections();});}
}
void main();
