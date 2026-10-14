import { Worker } from 'node:worker_threads';
import { performance } from 'node:perf_hooks';
function start(input: number): Promise<number> {
    return new Promise<number>((resolve, reject) => {
            const worker = new Worker("const {parentPort,workerData}=require('node:worker_threads');parentPort.postMessage(workerData*workerData);", { eval: true, workerData: input });
            let result: number | undefined;
            worker.once('message', (message: number) => { result = message; });
            worker.once('error', reject);
            worker.once('exit', (code) => {
                if (code !== 0 || result === undefined) reject(new Error(`Worker failed: ${code}`));
                else resolve(result);
            });
        });
}
async function batch(): Promise<number> {
    let checksum = 0;
    for (let input = 0; input < 8; input += 2) {
        const first = start(input);
        const second = start(input + 1);
        try {
            const [a, b] = await Promise.all([first, second]);
            checksum += a + b;
        } finally {
            await Promise.allSettled([first, second]);
        }
    }
    return checksum;
}
async function main() {
    for (let warmup = 0; warmup < 8; warmup++) await batch();
    for (let sample = 0; sample < 31; sample++) {
        const start = performance.now();
        const checksum = await batch();
        const elapsed = performance.now() - start;
        console.log(checksum);
        console.log(elapsed);
    }
}
main().catch(error => { console.error(error); process.exitCode = 1; });
