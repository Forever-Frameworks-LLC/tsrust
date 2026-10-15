import { Worker } from 'node:worker_threads';
import { performance } from 'node:perf_hooks';
function calculate(seed: number): number {
  const job = {seed};
  let value = job.seed;
  for (let step = 0; step < 250000; step += 1) value = (value * 48271) % 2147483647;
  const result = {kind: 'complete', value};
  return result.value;
}
async function measure(workers: number): Promise<void> {
  const pool = Array.from({ length: workers }, () => new Worker(`const {parentPort}=require('node:worker_threads'); const calculate=${calculate.toString()}; parentPort.on('message', seed=>parentPort.postMessage(calculate(seed)));`, { eval: true }));
  await Promise.all(pool.map(worker => new Promise<void>(resolve => worker.once('online', resolve))));
  const batch = async (offset: number): Promise<number> => {
    if (workers === 0) return [1, 2, 3, 4].reduce((sum, seed) => sum + calculate((seed - 1 + offset) % 4 + 1), 0);
    const values = await Promise.all(pool.map(async (worker, index) => {
      let sum = 0;
      for (let seed = index + 1; seed <= 4; seed += workers) {
        sum += await new Promise<number>((resolve, reject) => {
          const failure = (error: Error) => reject(error);
          worker.once('error', failure);
          worker.once('message', (value: number) => { worker.off('error', failure); resolve(value); });
          worker.postMessage((seed - 1 + offset) % 4 + 1);
        });
      }
      return sum;
    }));
    return values.reduce((sum, value) => sum + value, 0);
  };
  try {
    for (let warmup = 0; warmup < 8; warmup += 1) await batch(warmup);
    for (let sample = 0; sample < 31; sample += 1) {
      const start = performance.now(); const checksum = await batch(sample);
      const elapsed = performance.now() - start;
      console.log(workers); console.log(checksum); console.log(elapsed);
    }
  } finally { await Promise.all(pool.map(worker => worker.terminate())); }
}
async function main(): Promise<void> { for (const workers of [0, 2, 4]) await measure(workers); }
main().catch(error => { console.error(error); process.exitCode = 1; });
