import json,subprocess,hashlib,sys
from pathlib import Path
from datetime import datetime,timezone
root=Path('/Users/seancotter/Desktop/dev/learning/tsrust');out=root/'benchmarks/storage/closed-checkpoint-86-analytics';out.mkdir()
now=lambda:datetime.now(timezone.utc).isoformat()
def sha(p):
 h=hashlib.sha256()
 with Path(p).open('rb') as f:
  for block in iter(lambda:f.read(131072),b''):h.update(block)
 return h.hexdigest()
db=root/'benchmarks/records.sqlite';proof=root/'benchmarks/runtime/diagnostics/2026-10-03T13-57-02-000Z-checkpoint86-runtime-proof.json'
argv=['python3','-B',str(root/'benchmarks/storage/analytics.py'),'ingest',str(db),'--root',str(root/'benchmarks'),'--proof',str(proof),'--proof-sha256','da4c6d471e290fe5336640d83073dd4bad7a562a7f4670226cf09e8dbd9f2404']
log=out/'ingest.stdout';error=out/'ingest.stderr';started=now();before={'bytes':db.stat().st_size,'sha256':sha(db),'reportSha256':sha(root/'BENCHMARKS.md')}
with log.open('xb') as stdout,error.open('xb') as stderr:
 child=subprocess.Popen(argv,cwd=root,stdout=stdout,stderr=stderr);record={'argv':argv,'pid':child.pid,'startedUtc':started,'before':before};(out/'ingest-start.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps(record),flush=True);code=child.wait()
record.update({'exitCode':code,'finishedUtc':now(),'stdout':{'path':str(log),'sha256':sha(log)},'stderr':{'path':str(error),'sha256':sha(error)},'after':{'bytes':db.stat().st_size,'sha256':sha(db),'reportSha256':sha(root/'BENCHMARKS.md')}})
(out/'ingest-terminal.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps(record),flush=True);sys.exit(code)
