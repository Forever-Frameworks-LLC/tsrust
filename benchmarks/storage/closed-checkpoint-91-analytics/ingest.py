from pathlib import Path
import json,sqlite3,hashlib,subprocess,datetime
ROOT=Path('/Users/seancotter/Desktop/dev/learning/tsrust');DB=ROOT/'benchmarks/records.sqlite';D=ROOT/'benchmarks/storage/closed-checkpoint-91-analytics';D.mkdir(exist_ok=False)
def sha(p):
 h=hashlib.sha256()
 with Path(p).open('rb') as f:
  for block in iter(lambda:f.read(1048576),b''):h.update(block)
 return h.hexdigest()
def stats():
 db=sqlite3.connect('file:'+str(DB)+'?mode=ro',uri=True);d={t:db.execute('SELECT count(*) FROM '+t).fetchone()[0] for t in ['records','blobs','analytics_runs','analytics_samples','analytics_summaries','analytics_profiles']};db.close();return {'databaseBytes':DB.stat().st_size,**d}
now=lambda:datetime.datetime.now(datetime.timezone.utc).isoformat();before=stats();(D/'before-stats.json').write_text(json.dumps(before,indent=2)+'\n')
argv=['python3','-B',str(ROOT/'benchmarks/storage/analytics.py'),'ingest',str(DB),'--root',str(ROOT/'benchmarks'),'--proof',str(ROOT/'benchmarks/runtime/diagnostics/2026-10-03T18-11-33-000Z-checkpoint91-runtime-proof.json'),'--proof-sha256','aff99bf249cef51932388e9e7b59efe22578efba570beb1fa208e23aaaf6daac'];start=now()
with (D/'ingestion-stdout.json').open('xb') as out,(D/'ingestion-stderr.txt').open('xb') as err:
 p=subprocess.Popen(argv,stdout=out,stderr=err);(D/'ingestion-start.json').write_text(json.dumps({'argv':argv,'pid':p.pid,'startUTC':start},indent=2)+'\n');code=p.wait()
record={'argv':argv,'pid':p.pid,'startUTC':start,'finishUTC':now(),'exitCode':code,'stdoutSha256':sha(D/'ingestion-stdout.json'),'stderrSha256':sha(D/'ingestion-stderr.txt')};(D/'ingestion-terminal.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps(record),flush=True);assert code==0
(D/'post-ingestion-stats.json').write_text(json.dumps(stats(),indent=2)+'\n');print((D/'ingestion-stdout.json').read_text())
