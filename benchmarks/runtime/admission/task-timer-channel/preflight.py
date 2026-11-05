"""Bounded admission correctness only. Requires root-supplied exact graph artifacts.

No Cargo invocation, dependency fetch, registry mutation or feature timing.
The generated notification hook is disposable prototype source, never a change
to the compiler-owned runtime. Do not treat this as facade acceptance.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile

parser=argparse.ArgumentParser()
parser.add_argument('--toolchain',required=True)
parser.add_argument('--tokio',type=Path,required=True)
parser.add_argument('--tokio-util',type=Path,required=True)
parser.add_argument('--rayon',type=Path,required=True)
args=parser.parse_args()
here=Path(__file__).resolve().parent
root=here.parents[3]
artifacts={'tokio':args.tokio,'tokio_util':args.tokio_util,'rayon':args.rayon}
for name,path in artifacts.items():
    if not path.is_absolute() or not path.is_file():
        raise SystemExit(f'{name}: supply an existing absolute CargoJSON artifact path')

runtime=(root/'crates/tsrust-compiler/src/stdlib/threads/runtime.rs').read_text()
changes={
    'pool: Option<Arc<Pool>> }':'pool: Option<Arc<Pool>>, probe_done: Mutex<Option<tokio::sync::oneshot::Sender<()>>> }',
    'released: AtomicBool::new(false), pool })':'released: AtomicBool::new(false), pool, probe_done: Mutex::new(None) })',
    'completion.ready.notify_all();':'completion.ready.notify_all(); if let Some(done)=completion.probe_done.lock().unwrap().take(){let _=done.send(());}',
}
for before,after in changes.items():
    if runtime.count(before)!=1:
        raise SystemExit('Prototype hook source anchor changed: '+before)
    runtime=runtime.replace(before,after)
prefix,suffix=runtime.rsplit('}',1)
runtime=prefix+(here/'pool_transport.rs').read_text()+'}'+suffix
with tempfile.TemporaryDirectory(prefix='tsrust-task-timer-channel-') as temporary:
    temporary=Path(temporary)
    support=temporary/'pool.rs';support.write_text(runtime)
    binary=temporary/('probe.exe' if os.name=='nt' else 'probe')
    env=dict(os.environ,TSRUST_B06_THREADS_SOURCE=str(support),TSRUST_B06_FUTURES_SOURCE=str(root/'crates/tsrust-compiler/src/stdlib/futures/runtime.rs'))
    command=['rustup','run',args.toolchain,'rustc','--edition=2024','-O',str(here/'reference.rs'),'-o',str(binary)]
    for name,path in artifacts.items():command+=['--extern',f'{name}={path}']
    for path in sorted({path.parent for path in artifacts.values()}):command+=['-L',f'dependency={path}']
    subprocess.run(command,env=env,check=True,timeout=180)
    output=subprocess.run([str(binary)],env=env,check=True,capture_output=True,text=True,timeout=15)
    actual=json.loads(output.stdout)
    # Keep the independently authored oracle authoritative; use the pinned Node.
    oracle=temporary/'actual.json';oracle.write_text(json.dumps(actual))
    node='/Users/seancotter/.nvm/versions/node/v24.11.1/bin/node'
    subprocess.run([node,str(here/'oracle.mjs'),str(oracle)],check=True,timeout=10)
    print(json.dumps(actual,sort_keys=True))
