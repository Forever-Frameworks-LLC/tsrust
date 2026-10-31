// Latest native speed relative to Node.js after tsc, one line per workload.
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
const engine=process.argv.find(x=>x.startsWith('--engine='))?.slice(9)??'node-tsc';
const args=['-B',fileURLToPath(new URL('../storage/measurements.py',import.meta.url)),'latest','--engine',engine,...(process.argv.includes('--json')?['--json']:[])];
const result=spawnSync('python3',args,{encoding:'utf8',stdio:['ignore','inherit','pipe']});
if(result.error)throw result.error;if(result.status!==0)throw Error(result.stderr.trim());
