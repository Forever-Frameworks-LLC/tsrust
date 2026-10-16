declare function require(name: string): any;
const crypto = require('node:crypto');
const Buffer = require('node:buffer').Buffer;
async function batch(): Promise<number> {
  let sum = 0;
  const timestamp = new Date('1969-12-31T23:59:59.999Z');
  const expected = crypto.createHash('sha256').update('a'.repeat(64)).digest();
  for (let index = 0; index < 128; index++) {
    if (Math.floor(timestamp.getTime() / 1000) * 1000 !== -1000) throw new Error('Clock rounding mismatch');
    const digest = crypto.createHash('sha256').update('a'.repeat(64)).digest();
    if (!crypto.timingSafeEqual(digest, expected)) throw new Error('Token mismatch');
    const input = new Request('http://localhost/notes', { method: 'POST', body: 'a=one+two&a=%C3%A9', headers: { 'content-type': 'application/x-www-form-urlencoded' } });
    const fields = [...new URLSearchParams(await input.text()).entries()];
    if (fields.length !== 2 || fields[0][1] !== 'one two' || fields[1][1] !== 'é') throw new Error('Form mismatch');
    const log = JSON.stringify({ operation: 'notes.write', status: 200, bytes: 9 });
    if (log !== '{"operation":"notes.write","status":200,"bytes":9}') throw new Error('Log mismatch');
    sum += digest.length + Buffer.byteLength(fields[0][1]) + Buffer.byteLength(fields[1][1]) + 1;
  }
  return sum;
}
async function main(): Promise<void> {
  for (let warmup = 0; warmup < 8; warmup++) await batch();
  for (let sample = 0; sample < 31; sample++) { const start = performance.now(); const sum = await batch(); console.log(sum); console.log(performance.now() - start); }
}
void main();
