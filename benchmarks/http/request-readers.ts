declare function require(name: string): any;
const http = require('node:http');
const Buffer = require('node:buffer').Buffer;
async function batch(url: string): Promise<number> {
  let sum = 0;
  for (let index = 0; index < 128; index++) {
    const json = await fetch(url + 'json', {method: 'POST', body: '{"value":40}'});
    if (json.status !== 200 || await json.text() !== '40') throw new Error('JSON response mismatch');
    const stream = await fetch(url + 'stream', {method: 'POST', body: 'é'});
    if (stream.status !== 200 || await stream.text() !== '2') throw new Error('Stream response mismatch');
    sum += 40 + 2;
  }
  return sum;
}
async function main(): Promise<void> {
  const server = http.createServer(async (input: any, response: any) => {
    const chunks = [];
    let bytes = 0;
    for await (const chunk of input) {
      bytes += chunk.length;
      if (bytes > 1048576) throw new Error('Body limit');
      chunks.push(chunk);
    }
    const text = new TextDecoder('utf-8', {fatal: true}).decode(Buffer.concat(chunks));
    if (input.method !== 'POST') throw new Error('Method mismatch');
    let result;
    if (input.url === '/json') {
      const value: unknown = JSON.parse(text);
      if (typeof value !== 'object' || value === null || !('value' in value) || !Number.isSafeInteger(value.value) || value.value !== 40) throw new Error('JSON mismatch');
      result = value.value;
    } else {
      if (input.url !== '/stream' || text !== 'é') throw new Error('Stream mismatch');
      result = Buffer.byteLength(text);
    }
    response.setHeader('content-type', 'text/plain; charset=utf-8');
    response.end(String(result));
  });
  await new Promise<void>(resolve => server.listen(0, '127.0.0.1', resolve));
  const url = `http://127.0.0.1:${server.address().port}/`;
  try {
    for (let warmup = 0; warmup < 8; warmup++) await batch(url);
    for (let sample = 0; sample < 31; sample++) {const start = performance.now();const sum = await batch(url);console.log(sum);console.log(performance.now() - start);}
  } finally {await new Promise<void>(resolve => {server.close(resolve);server.closeIdleConnections();});}
}
void main();
