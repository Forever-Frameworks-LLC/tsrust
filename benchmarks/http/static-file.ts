declare function require(name: string): any;
declare const process: { env: Record<string, string | undefined> };
const http = require('node:http');
const fs = require('node:fs');
const path = require('node:path');
async function batch(url: string): Promise<number> {
  let sum = 0;
  for (let index = 0; index < 128; index++) {
    const response = await fetch(url);
    if (response.status !== 200 || await response.text() !== '42') throw new Error('Unexpected static response');
    sum += 42;
  }
  return sum;
}
async function main(): Promise<void> {
  const root = process.env.TSRUST_STATIC_BENCH_ROOT;
  if (!root) throw new Error('Missing static benchmark root');
  const file = path.join(root, 'value.txt');
  const server = http.createServer((request: any, response: any) => {
    if (request.method !== 'GET' || request.url !== '/assets/value.txt') { response.writeHead(404); response.end(); return; }
    fs.stat(file, (error: unknown, metadata: any) => {
      if (error) { response.writeHead(500); response.end(); return; }
      response.writeHead(200, { 'content-type': 'text/plain', 'content-length': metadata.size, 'last-modified': metadata.mtime.toUTCString() });
      const stream = fs.createReadStream(file);
      stream.on('error', () => response.destroy());
      response.on('close', () => stream.destroy());
      stream.pipe(response);
    });
  });
  await new Promise<void>(resolve => server.listen(0, '127.0.0.1', resolve));
  const url = `http://127.0.0.1:${server.address().port}/assets/value.txt`;
  try {
    for (let warmup = 0; warmup < 8; warmup++) await batch(url);
    for (let sample = 0; sample < 31; sample++) { const start = performance.now(); const sum = await batch(url); console.log(sum); console.log(performance.now() - start); }
  } finally { await new Promise<void>(resolve => { server.close(resolve); server.closeIdleConnections(); }); }
}
void main();
