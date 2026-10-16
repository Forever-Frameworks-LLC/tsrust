declare function require(name: string): any;
const { Readable } = require('node:stream');
const { Buffer } = require('node:buffer');
async function* produce(): AsyncGenerator<any> {
  for (let index = 0; index < 6; index++) yield Buffer.from('abcdefg');
}
async function batch(): Promise<number> {
  let sum = 0;
  for (let index = 0; index < 128; index++) {
    const chunks: any[] = [];
    for await (const chunk of Readable.from(produce(), {highWaterMark: 1})) chunks.push(chunk);
    const body = Buffer.concat(chunks).toString('utf8');
    if (body !== 'abcdefgabcdefgabcdefgabcdefgabcdefgabcdefg') throw new Error('Unexpected body');
    sum += body.length;
  }
  return sum;
}
async function main(): Promise<void> {
  for (let warmup = 0; warmup < 8; warmup++) await batch();
  for (let sample = 0; sample < 31; sample++) {
    const start = performance.now();
    const sum = await batch();
    console.log(sum);
    console.log(performance.now() - start);
  }
}
void main();
