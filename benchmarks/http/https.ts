declare function require(name: string): any;
const http2 = require('node:http2');
const fs = require('node:fs');
const pem = (name: string): string => fs.readFileSync('crates/tsrust-compiler/tests/fixtures/tls/' + name, 'utf8');
async function batch(client: any): Promise<number> {
  let sum = 0;
  for (let index = 0; index < 128; index++) {
    const body = await new Promise<string>((resolve, reject) => {
      const stream = client.request({':method': 'GET', ':path': '/'});
      let body = '';stream.setEncoding('utf8');
      stream.on('response', (headers: any) => {if (headers[':status'] !== 200) reject(new Error('Unexpected status'));});
      stream.on('data', (chunk: string) => {body += chunk;});
      stream.on('end', () => resolve(body));stream.on('error', reject);
      stream.setTimeout(30000, () => stream.destroy(new Error('Timeout')));stream.end();
    });
    if (body !== '42') throw new Error('Unexpected body');
    sum += 42;
  }
  return sum;
}
async function main(): Promise<void> {
  const server = http2.createSecureServer({cert: pem('server.pem'), key: pem('server.key')});
  server.on('stream', (stream: any) => {stream.respond({':status': 200, 'content-type': 'text/plain; charset=utf-8'});stream.end('42');});
  await new Promise<void>(resolve => server.listen(0, '127.0.0.1', resolve));
  const client = http2.connect(`https://localhost:${server.address().port}`, {ca: pem('ca.pem'), family: 4});
  try {
    for (let warmup = 0; warmup < 8; warmup++) await batch(client);
    for (let sample = 0; sample < 31; sample++) {const start = performance.now();const sum = await batch(client);console.log(sum);console.log(performance.now() - start);}
  } finally {client.destroy();await new Promise<void>(resolve => server.close(resolve));}
}
void main();
