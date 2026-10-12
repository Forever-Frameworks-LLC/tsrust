declare const process:{argv:string[]};
const n = Number(process.argv[2]);
// Keep adapter's two call sites; the untaken branch does not warm the process.
const value:unknown=n<0?workload(0, 7):workload(n, 7);
const checksum=probeChecksum(value);
(globalThis as unknown as {retained:unknown}).retained=value;
console.log(JSON.stringify({checksum}));
