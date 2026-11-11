export function rustReferenceSource(workload) {
 const source=workload.rustReference===undefined?`parity/${workload.id}.rs`:workload.rustReference;
 if(typeof source!=='string'||!/^parity\/[A-Za-z0-9_-]+\.rs$/.test(source))throw new Error(`Invalid hand-written Rust source for ${workload.id}`);
 return source;
}
