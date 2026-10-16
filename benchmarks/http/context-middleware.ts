declare function require(name: string): any;
const http = require('node:http');
async function batch(url: string): Promise<number> {
  let sum = 0;
  for (let index = 0; index < 128; index++) {
    const response = await fetch(url, { redirect: 'manual' });
    if (response.headers.get('x-before') !== 'entry' || response.headers.get('x-after') !== 'exit') throw new Error('Middleware header mismatch');
    if (response.status !== 200 || await response.text() !== '42') throw new Error('Unexpected response');
    sum += 42;
  }
  return sum;
}
async function main(): Promise<void> {
  const config: Readonly<{body:string}> = {body:'42'};
  const server = http.createServer(async (_request: any, response: any) => {
    const context = { env: config, variables: new Map<string,string>(), headers: new Map<string,string>(), body: '' };
    async function handle(): Promise<void> { const body = context.variables.get('body'); if (body === undefined) throw new Error('Missing request variable'); context.body = body; }
    async function next(): Promise<void> { await handle(); }
    context.headers.set('x-before', 'entry');
    context.variables.set('body', context.env.body);
    await next();
    context.headers.set('x-after', 'exit');
    response.setHeader('content-type', 'text/plain; charset=utf-8');
    for (const [name,value] of context.headers) response.setHeader(name,value);
    response.end(context.body);
  });
  await new Promise<void>(resolve => server.listen(0, '127.0.0.1', resolve));
  const url = `http://127.0.0.1:${server.address().port}/`;
  try {
    for (let warmup = 0; warmup < 8; warmup++) await batch(url);
    for (let sample = 0; sample < 31; sample++) {const start = performance.now();const sum = await batch(url);console.log(sum);console.log(performance.now() - start);}
  } finally {await new Promise<void>(resolve => {server.close(resolve);server.closeIdleConnections();});}
}
void main();
