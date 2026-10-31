declare function require(name: string): any;
declare const process: any;
const http = require('node:http');
function render(id: number): { id: number; checksum: number; escaped: number } {
  let checksum = 0, escaped = 0;
  for (let y = 0; y < 64; y++) for (let x = 0; x < 96; x++) {
    const cx = -2.0 + x * 3.0 / 96.0 + (id % 7) * 0.0001;
    const cy = -1.0 + y * 2.0 / 64.0 + (id % 11) * 0.0001;
    let zx = 0.0, zy = 0.0, iterations = 0;
    while (iterations < 256 && zx * zx + zy * zy <= 4.0) {
      const next = zx * zx - zy * zy + cx;
      zy = 2.0 * zx * zy + cy; zx = next; iterations++;
    }
    checksum = (checksum + iterations * (y * 96 + x + 1)) % 1000000007;
    if (iterations < 256) escaped++;
  }
  return { checksum, escaped, id };
}
const decoder = new TextDecoder('utf-8', { fatal: true });
function respond(response: any, status: number, type: string, body: string): void {
  response.writeHead(status, { 'content-type': type }); response.end(body);
}
const server = http.createServer(async (request: any, response: any) => {
  if (request.method !== 'POST' || request.url !== '/render') return respond(response, 404, 'text/plain', 'missing');
  let id: number;
  try {
    const chunks: Uint8Array[] = []; let length = 0;
    for await (const chunk of request) { length += chunk.length; if (length > 65536) throw Error('body limit'); chunks.push(chunk); }
    const buffer = new Uint8Array(length); let offset = 0;
    for (const chunk of chunks) { buffer.set(chunk, offset); offset += chunk.length; }
    const input = JSON.parse(decoder.decode(buffer));
    if (input === null || typeof input !== 'object' || Array.isArray(input) || !Number.isSafeInteger(input.id) || input.id < 1 || input.id > 128) throw Error('input');
    id = input.id;
  } catch { return respond(response, 400, 'text/plain', 'invalid'); }
  respond(response, 200, 'application/json', JSON.stringify(render(id)));
});
server.maxConnections = 64; server.requestTimeout = 5000;
server.listen(0, '127.0.0.1', () => console.log(JSON.stringify({ address: '127.0.0.1:' + server.address().port })));
process.on('SIGINT', () => server.close());
