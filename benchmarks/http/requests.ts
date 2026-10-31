declare function require(name: string): any;
declare const process: any;
const http = require('node:http');
const { DatabaseSync } = require('node:sqlite');
const db = new DatabaseSync(':memory:');
db.exec('CREATE TABLE items(id INTEGER PRIMARY KEY, label TEXT NOT NULL, value INTEGER NOT NULL)');
const insert = db.prepare('INSERT INTO items VALUES (?, ?, ?)');
for (let id = 1; id <= 128; id++) insert.run(id, `row héllo ${id}`, id * 3);
const query = db.prepare('SELECT id, label, value FROM items WHERE id = ?');
const decoder = new TextDecoder('utf-8', { fatal: true });
function send(response: any, status: number, type: string, body: string): void {
  response.writeHead(status, { 'content-type': type });
  response.end(body);
}
function text(response: any, status: number, body: string): void { send(response, status, 'text/plain; charset=utf-8', body); }
function json(response: any, value: unknown): void { send(response, 200, 'application/json', JSON.stringify(value)); }
async function body(request: any): Promise<any> {
  const chunks: Uint8Array[] = []; let length = 0;
  for await (const chunk of request) {
    length += chunk.length;
    if (length > 65536) throw Error('body limit');
    chunks.push(chunk);
  }
  const buffer = new Uint8Array(length); let offset = 0;
  for (const chunk of chunks) { buffer.set(chunk, offset); offset += chunk.length; }
  return JSON.parse(decoder.decode(buffer));
}
function identifier(value: unknown): value is number { return typeof value === 'number' && Number.isSafeInteger(value) && value >= 1 && value <= 1000000; }
const server = http.createServer(async (request: any, response: any) => {
  if (request.method === 'GET' && request.url === '/text') return text(response, 200, 'héllo 🦀');
  if (request.method === 'GET' && request.url === '/json') return json(response, { id: 7, label: 'héllo 🦀', value: 42 });
  if (request.method !== 'POST' || !['/parse', '/sqlite'].includes(request.url)) return text(response, 404, 'missing');
  let value: any;
  try {
    value = await body(request);
    if (value === null || typeof value !== 'object' || Array.isArray(value) || !identifier(value.id)) throw Error('shape');
    if (request.url === '/parse' && (typeof value.label !== 'string' || !value.label.isWellFormed() || typeof value.value !== 'number' || !Number.isSafeInteger(value.value) || value.value < 0 || value.value > 1000000)) throw Error('shape');
  } catch { return text(response, 400, 'invalid'); }
  if (request.url === '/parse') return json(response, { id: value.id, label: value.label + '!', value: value.value + 1 });
  const row = query.get(value.id);
  if (row === undefined) return text(response, 404, 'missing');
  return json(response, { id: row.id, label: row.label, value: row.value });
});
server.maxConnections = 64; server.requestTimeout = 5000;
server.listen(0, '127.0.0.1', () => console.log(JSON.stringify({ address: `127.0.0.1:${server.address().port}`, sqliteVersion: db.prepare('SELECT sqlite_version() AS version').get().version })));
process.once('SIGINT', () => server.close(() => { db.close(); }));
