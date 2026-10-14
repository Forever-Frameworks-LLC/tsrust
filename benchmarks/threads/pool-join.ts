import { Worker } from 'node:worker_threads';
import { performance } from 'node:perf_hooks';

async function main() {
    const workers = [0, 1].map(() => new Worker("const {parentPort}=require('node:worker_threads');parentPort.on('message',n=>parentPort.postMessage(n*n));", { eval: true }));
    await Promise.all(workers.map(worker => new Promise<void>((resolve, reject) => {
        worker.once('online', resolve); worker.once('error', reject);
    })));
    const run = (worker: Worker, value: number) => new Promise<number>((resolve, reject) => {
        const fail = (error: Error) => { worker.off('message', complete); reject(error); };
        const complete = (result: number) => { worker.off('error', fail); resolve(result); };
        worker.once('message', complete); worker.once('error', fail); worker.postMessage(value);
    });
    async function batch(): Promise<number> {
        let checksum = 0;
        for (let index = 0; index < 8; index += 2) {
            const [left, right] = await Promise.all([run(workers[0], index), run(workers[1], index + 1)]);
            checksum += left + right;
        }
        return checksum;
    }
    try {
        for (let warmup = 0; warmup < 8; warmup++) await batch();
        for (let sample = 0; sample < 31; sample++) {
            const start = performance.now();
            const checksum = await batch();
            console.log(checksum); console.log(performance.now() - start);
        }
    } finally { await Promise.all(workers.map(worker => worker.terminate())); }
}
main().catch(error => { console.error(error); process.exitCode = 1; });
