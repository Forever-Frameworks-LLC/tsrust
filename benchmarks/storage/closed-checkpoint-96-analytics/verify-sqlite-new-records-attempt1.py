from pathlib import Path
import json,sqlite3,hashlib,tempfile,shutil,sys,datetime,math
R=Path('/Users/seancotter/Desktop/dev/learning/tsrust');P=Path('/tmp/tsrust-checkpoint96-plan');D=R/'benchmarks/storage/closed-checkpoint-96-analytics';DB=R/'benchmarks/records.sqlite';sys.path.insert(0,str(R/'benchmarks/storage'));import store
load=lambda p:json.loads(Path(p).read_bytes())
def sha(p):
 h=hashlib.sha256()
 with Path(p).open('rb') as f:
  for block in iter(lambda:f.read(1048576),b''):h.update(block)
 return h.hexdigest()
plan=load(P/'sqlite-import-plan.json');assert sha(P/'sqlite-import-plan.json')=='a1b0faa705113b654383179a3e4734dfea1e6b53302eca2e10415ba023de430c';assert sha(plan['proof']['path'])==plan['proof']['sha256'];before=load(D/'before-stats.json');scratch=Path(tempfile.mkdtemp(prefix='tsrust-checkpoint96-exact-restore-'));records=[]
for item in plan['records']:
 source=Path(item['path']);relative=source.relative_to(R/'benchmarks').as_posix();store.export_record(DB,relative,scratch);restored=scratch/relative
 assert sha(source)==sha(restored)==item['sha256'];assert source.stat().st_size==restored.stat().st_size==item['bytes']
 assert sha(item['originalMeasurementPath'])==item['sha256']
 records.append({'relativePath':relative,'sha256':item['sha256'],'bytes':item['bytes'],'originalExact':True,'snapshotOriginalExact':True,'restoredExact':True})
assert {p.relative_to(scratch).as_posix() for p in scratch.rglob('*') if p.is_file()}=={r['relativePath'] for r in records}
db=sqlite3.connect('file:'+str(DB)+'?mode=ro',uri=True);stats={'databaseBytes':DB.stat().st_size,**{t:db.execute('SELECT count(*) FROM '+t).fetchone()[0] for t in ['records','blobs','analytics_runs','analytics_samples','analytics_summaries','analytics_profiles']}}
for r in records:
 row=db.execute('SELECT records.sha256,original_size,length(compressed) FROM records JOIN blobs USING(sha256) WHERE path=?',(r['relativePath'],)).fetchone();assert row[:2]==(r['sha256'],r['bytes']);r['compressedBytes']=row[2]
raw=plan['records'][0]['sha256'];prof=plan['records'][1]['sha256'];assert db.execute('SELECT count(*) FROM analytics_samples WHERE raw_sha=?',(raw,)).fetchone()[0]==8928;assert db.execute('SELECT count(*) FROM analytics_summaries WHERE raw_sha=?',(raw,)).fetchone()[0]==96;assert db.execute('SELECT count(*) FROM analytics_profiles WHERE raw_sha=?',(prof,)).fetchone()[0]==144
cells=db.execute('SELECT workload,engine,process_index,count(*),min(sample_index),max(sample_index),min(operations),max(operations) FROM analytics_samples WHERE raw_sha=? GROUP BY workload,engine,process_index',(raw,)).fetchall();assert len(cells)==288 and all(row[3:]==(31,0,30,2048,2048) for row in cells)
assert db.execute('SELECT count(*) FROM analytics_profiles WHERE raw_sha=? AND (live_after_output!=0 OR live_after_drop!=0)',(prof,)).fetchone()[0]==0
query=load(D/'checkpoint-query.log');comparison=load(P/'verification/comparison.json');assert query['comparedRows']==48 and query['nativeWins']==27
for row in query['rows']:
 original=next(x for x in comparison['rows'] if x['id']==row['id']);assert math.isclose(row['nativeNodeRatio'],original['nativeNodeRatio'],rel_tol=1e-12)
 for engine,key in [('native','native'),('node-tsc','compiledNode')]:
  for metric in ['min','avg','max']:assert math.isclose(row[key][metric],original['statistics'][engine][metric],rel_tol=1e-12)
db.close();result={'gate':96,'acceptedOriginalBaselineIndexed':True,'before':before,'after':stats,'deltas':{k:stats[k]-before[k] for k in stats},'proof':plan['proof'],'records':records,'scratchRoot':str(scratch),'restoredPathSetExact':True,'independentRestoredHashesExact':True,'sqliteProcessSampleBoundariesExact':True,'sqliteRateSummariesMatchIndependentReciprocalDerivation':True,'profileDisposalsZero':True,'query':{'path':str(D/'checkpoint-query.log'),'sha256':sha(D/'checkpoint-query.log'),'comparedRows':48,'nativeWins':27,'reviewSlower':21},'looseOriginalsRemoved':False,'priorHistoryReverified':False,'at':datetime.datetime.now(datetime.timezone.utc).isoformat()};shutil.rmtree(scratch);result['verifiedScratchCopiesRemoved']=True;(D/'verified-import.json').write_text(json.dumps(result,indent=2)+'\n');(D/'post-ingestion-stats.json').write_text(json.dumps(stats,indent=2)+'\n');print(json.dumps({'verifiedImport':{'path':str(D/'verified-import.json'),'sha256':sha(D/'verified-import.json')},'stats':stats,'deltas':result['deltas']},indent=2))
