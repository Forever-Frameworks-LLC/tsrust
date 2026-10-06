import hashlib,json,os,shutil,sqlite3,sys,tempfile
from pathlib import Path
from datetime import datetime,timezone
root=Path('/Users/seancotter/Desktop/dev/learning/tsrust');bench=root/'benchmarks';out=bench/'storage/closed-checkpoint-86-analytics';sys.path.insert(0,str(bench/'storage'));import store
proof=bench/'runtime/diagnostics/2026-10-03T13-57-02-000Z-checkpoint86-runtime-proof.json';certificate=Path('/tmp/tsrust-gate86/completed-acceptance.json')
expected=['runtime/results/2026-10-03T13-42-38-756Z.json','runtime/diagnostics/2026-10-03T13-46-03-175Z-native-allocations.json'];db=bench/'records.sqlite'
def sha(p):
 h=hashlib.sha256()
 with Path(p).open('rb') as f:
  for block in iter(lambda:f.read(131072),b''):h.update(block)
 return h.hexdigest()
def save(name,value):
 with (out/name).open('x') as f:json.dump(value,f,indent=2);f.write('\n')
def protected():return {str(p.relative_to(root)):(p.stat().st_dev,p.stat().st_ino,p.stat().st_size,p.stat().st_mtime_ns,p.stat().st_ctime_ns) for folder in ['results','diagnostics'] for p in (bench/'runtime'/folder).glob('*.json')}
c=json.loads(certificate.read_bytes());assert c['gate']==86 and all(c[k] is True for k in ['regressionAccepted','runtimeAccepted','profilesAccepted','documentationAccepted','allConsumersFinished'])
assert sha(proof)=='da4c6d471e290fe5336640d83073dd4bad7a562a7f4670226cf09e8dbd9f2404'
records=json.loads(proof.read_bytes())['records'];assert [Path(x['path']).relative_to(bench).as_posix() for x in records]==expected
before=protected();report_sha=sha(root/'BENCHMARKS.md');scratch=Path(tempfile.mkdtemp(prefix='tsrust-86-raw-restored-'));checks=[];removed=[]
try:
 for name,item in zip(expected,records):
  p=bench/name;stat=p.stat();original=sha(p);assert original==item['sha256']
  archived=list(store.list_records(db,name));assert len(archived)==1 and archived[0]['path']==name and archived[0]['sha256']==original and archived[0]['original_size']==stat.st_size
  store.export_record(db,name,scratch);restored=sha(scratch/name);assert restored==original and (scratch/name).stat().st_size==stat.st_size
  checks.append({'path':name,'originalSha256':original,'restoredSha256':restored,'archiveSha256':archived[0]['sha256'],'bytes':stat.st_size,'allocatedBytes':stat.st_blocks*512,'restorationVerified':True})
 save('raw-restore-verification.json',{'gate':86,'timestamp':datetime.now(timezone.utc).isoformat(),'certificate':{'path':str(certificate),'sha256':sha(certificate)},'runtimeProof':{'path':str(proof),'sha256':sha(proof)},'records':checks,'independentRestoredSha256Compared':True,'selectedPathsExact':True,'rawReencoded':False,'scratch':str(scratch)})
 for check in checks:
  p=bench/check['path'];assert sha(p)==check['originalSha256'];p.unlink();removed.append(check['path'])
 after=protected();assert after=={key:value for key,value in before.items() if key not in {'benchmarks/'+name for name in expected}}
 assert sha(root/'BENCHMARKS.md')==report_sha
except BaseException:
 for name in removed:store.export_record(db,name,bench)
 raise
else:shutil.rmtree(scratch)
prior=json.loads((bench/'storage/closed-checkpoint-86-migration/migration.json').read_bytes());allocated=sum(x['allocatedBytes'] for x in checks);growth=db.stat().st_blocks*512-prior['database']['allocatedBytes']
with store.connect(db) as sql:
 counts={name:sql.execute('SELECT count(*) FROM '+name).fetchone()[0] for name in ['records','blobs','analytics_runs','analytics_samples','analytics_summaries','analytics_profiles']}
actual={'gate':86,'completedRawRecordsRemoved':2,'removedPaths':expected,'exactOriginalBytesRestorable':True,'protectedRemainingLooseJsonPathStatIdentitiesExact':True,'reportSha256':report_sha,'reportUnchanged':True,'database':{'path':str(db),'sha256':sha(db),'bytes':db.stat().st_size,'allocatedBytes':db.stat().st_blocks*512,**counts},'originalLogicalBytes':sum(x['bytes'] for x in checks),'originalAllocatedBytes':allocated,'databaseLogicalGrowthBytes':db.stat().st_size-134262784,'databaseAllocatedGrowthBytes':growth,'grossAllocatedBytesReclaimed':allocated-growth,'scratchVerifiedCopiesRemoved':True,'oldRawHistoryMigrated':False,'newGate87EvidenceTouched':False}
save('post-ingestion-stats.json',actual);print(json.dumps(actual))
