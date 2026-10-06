// Concise accepted SQLite checkpoint comparison; raw bytes remain in the exact archive.
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
const database=process.argv.find(x=>x.startsWith('--database='))?.slice(11)??fileURLToPath(new URL('../records.sqlite',import.meta.url));
const raw=process.argv.find(x=>x.startsWith('--raw-sha256='))?.slice(13);
const args=['-B',fileURLToPath(new URL('../storage/analytics.py',import.meta.url)),'checkpoint',database,...(raw?['--raw-sha256',raw]:[])];
const result=spawnSync('python3',args,{encoding:'utf8',maxBuffer:16*1024*1024});
if(result.error)throw result.error;if(result.status!==0)throw Error(result.stderr.trim());
const data=JSON.parse(result.stdout);
if(process.argv.includes('--json'))console.log(JSON.stringify(data,null,2));
else{
 console.log(`${data.timestamp} · ${data.semanticEpoch} · ${data.measurementScope}`);
 console.log(`${data.comparedRows} source-matched rows: native faster ${data.nativeWins}, review slower ${data.rows.filter(x=>x.nativeNodeRatio<1).length}. Same accepted run; complete workload rates.`);
 for(const row of data.rows)console.log(`${row.id}: native ${row.native.avg.toFixed(2)} / compiled Node ${row.compiledNode.avg.toFixed(2)} ops/s · ${row.nativeNodeRatio.toFixed(3)}×${row.nativeNodeRatio<1?' · review':''}`);
}
