from pathlib import Path
import json,hashlib,subprocess,datetime,sqlite3,shutil
ROOT=Path('/Users/seancotter/Desktop/dev/learning/tsrust');B=ROOT/'benchmarks';D=B/'storage/closed-checkpoint-91-analytics';DB=B/'records.sqlite';scratch=Path('/tmp/tsrust-gate91-raw-restore');scratch.mkdir(exist_ok=False)
def sha(p):
 h=hashlib.sha256()
 with p.open('rb') as stream:
  for block in iter(lambda:stream.read(1048576),b''):h.update(block)
 return h.hexdigest()
now=lambda:datetime.datetime.now(datetime.timezone.utc).isoformat();proof=ROOT/'benchmarks/runtime/diagnostics/2026-10-03T18-11-33-000Z-checkpoint91-runtime-proof.json';accepted=json.loads(proof.read_text());records=accepted['records'];assert len(records)==2
protected={str(p.relative_to(B)):{'bytes':p.stat().st_size,'mtimeNs':p.stat().st_mtime_ns,'inode':p.stat().st_ino} for folder in ['runtime/results','runtime/diagnostics'] for p in (B/folder).glob('*.json')};reportSHA=sha(ROOT/'BENCHMARKS.md');(D/'protected-before.json').write_text(json.dumps(protected,indent=2)+'\n')
argv=['/Users/seancotter/.nvm/versions/node/v24.11.1/bin/node',str(B/'runtime/checkpoint.mjs'),'--raw-sha256='+records[0]['sha256'],'--json'];start=now()
with (D/'checkpoint-query.json').open('xb') as out,(D/'checkpoint-query-stderr.txt').open('xb') as err:
 child=subprocess.Popen(argv,stdout=out,stderr=err);code=child.wait()
(D/'checkpoint-query-terminal.json').write_text(json.dumps({'argv':argv,'pid':child.pid,'startedUtc':start,'finishedUtc':now(),'exitCode':code,'stdoutSha256':sha(D/'checkpoint-query.json'),'stderrSha256':sha(D/'checkpoint-query-stderr.txt')},indent=2)+'\n');assert code==0
query=json.loads((D/'checkpoint-query.json').read_text());assert query['comparedRows']==68 and query['nativeWins']==34
commands=[];verified=[];db=sqlite3.connect('file:'+str(DB)+'?mode=ro',uri=True)
for b in records:
 original=Path(b['path']);rel=original.relative_to(B);assert sha(original)==b['sha256'];indexed=db.execute('SELECT records.path, records.sha256,blobs.original_size FROM records JOIN blobs USING(sha256) WHERE records.path=?',(str(rel),)).fetchone();assert indexed==(str(rel),b['sha256'],original.stat().st_size)
 argv=['python3','-B',str(B/'storage/store.py'),'export',str(DB),str(rel),'--root',str(scratch)];start=now()
 with (D/('restore-'+rel.name+'-stdout.txt')).open('xb') as out,(D/('restore-'+rel.name+'-stderr.txt')).open('xb') as err:
  child=subprocess.Popen(argv,stdout=out,stderr=err);code=child.wait()
 commands.append({'argv':argv,'pid':child.pid,'startedUtc':start,'finishedUtc':now(),'exitCode':code});assert code==0
 restored=scratch/rel;assert restored.stat().st_size==original.stat().st_size and sha(restored)==sha(original)==indexed[1]
 verified.append({'originalPath':str(original),'relativePath':str(rel),'sha256':b['sha256'],'restoredPath':str(restored),'bytes':original.stat().st_size,'allocatedBytes':original.stat().st_blocks*512,'restoredExact':True,'storeBindingExact':True})
db.close();assert {str(p.relative_to(scratch)) for p in scratch.rglob('*') if p.is_file()}=={r['relativePath'] for r in verified}
# Finish every restoration/hash before removing either original. Immediate fresh original hashes mandatory.
for b in verified:assert sha(Path(b['originalPath']))==sha(Path(b['restoredPath']))==b['sha256']
for b in verified:
 original=Path(b['originalPath']);assert sha(original)==b['sha256'];original.unlink()
remaining={str(p.relative_to(B)):{'bytes':p.stat().st_size,'mtimeNs':p.stat().st_mtime_ns,'inode':p.stat().st_ino} for folder in ['runtime/results','runtime/diagnostics'] for p in (B/folder).glob('*.json')};expected={k:v for k,v in protected.items() if k not in {r['relativePath'] for r in verified}};assert remaining==expected and sha(ROOT/'BENCHMARKS.md')==reportSHA
for b in verified:
 restored=Path(b['restoredPath']);assert sha(restored)==b['sha256'];restored.unlink()
for p in sorted([p for p in scratch.rglob('*') if p.is_dir()],key=lambda p:len(p.parts),reverse=True):p.rmdir()
scratch.rmdir()
before=json.loads((D/'before-stats.json').read_text());post=json.loads((D/'post-ingestion-stats.json').read_text());growth=post['databaseBytes']-before['databaseBytes'];gross=sum(b['allocatedBytes'] for b in verified)-growth
record={'gate':91,'accepted':True,'timed':True,'measurementRepeated':False,'sqliteCanonical':True,'proof':{'path':str(proof),'sha256':sha(proof)},'records':verified,'restoreCommands':commands,'allRestoredBeforeAnyOriginalRemoved':True,'exactOriginalRestoredStorePathsetsSizeSHA':True,'looseOriginalsRemoved':True,'scratchVerifiedCopiesRemoved':True,'protectedOriginalLooseRecords':len(expected),'protectedPreviousStatIdentitiesAndPathsetExact':True,'reportSha256':reportSHA,'before':before,'after':post,'databaseSha256':sha(DB),'databaseGrowthBytes':growth,'originalLogicalBytes':sum(b['bytes'] for b in verified),'originalAllocatedBytes':sum(b['allocatedBytes'] for b in verified),'grossAllocatedReclaimedBytes':gross,'checkpointComparedRows':68,'nativeWins':34,'reviewSlowerRows':34,'restoration':'python3 benchmarks/storage/store.py export benchmarks/records.sqlite RELATIVE_RECORD_PATH --root benchmarks; use scratch root for revalidation, existing outputs never overwritten.'}
(D/'migration.json').write_text(json.dumps(record,indent=2)+'\n');evidence=sum(p.stat().st_blocks*512 for p in D.iterdir() if p.is_file());record.update({'evidenceAllocatedBytes':evidence,'netAllocatedReclaimedBytes':gross-evidence});(D/'postmigration-stats.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k] for k in ['databaseGrowthBytes','originalLogicalBytes','grossAllocatedReclaimedBytes','netAllocatedReclaimedBytes','databaseSha256','after']}))
