// Verify collected diagnostic evidence without repeating the browser run.
import assert from 'node:assert/strict';
import fs from 'node:fs';
const [file,budget]=process.argv.slice(2);const result=JSON.parse(fs.readFileSync(file,'utf8'));const maximum=Number(budget);
assert(Number.isFinite(maximum)&&maximum>0,'Pass an explicit maximum event-loop gap in milliseconds');
assert(Number.isFinite(result.maxGap)&&result.maxGap<maximum,`Event-loop gap ${result.maxGap} ms exceeds ${maximum} ms`);
assert(result.nodes>0&&result.headings>0);assert.deepEqual(result.errors,[]);assert(result.wasm.some(value=>value.method==='instantiateStreaming'));
console.log(JSON.stringify({case:'startup-diagnostic-budget',status:'pass',eventLoopGapMs:result.maxGap,maximumMs:maximum,cpuRate:result.cpuRate,networkKbps:result.networkKbps||0,benchmark:false}));
