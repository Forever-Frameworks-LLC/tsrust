from pathlib import Path
import json,hashlib,importlib.util,datetime,time,zlib,os
root=Path('/Users/seancotter/Desktop/dev/learning/tsrust')
out=Path('/tmp/tsrust-gate99-bundle-storage')
index=root/'benchmarks/runtime/diagnostics/gate99-closure-evidence/restoration-map.json'
assert index.is_file(), 'Root-approved final Gate99 bundle must be packaged first'
manifest=json.loads(index.read_text())
assert manifest.get('readyForImport') is True and manifest.get('gate')==99, 'Read-only provisional inventory cannot be imported'
for item in manifest['records']:
 p=root/item['storedPath']; assert p.is_file() and hashlib.sha256(p.read_bytes()).hexdigest()==item['sha256'] and p.stat().st_size==item['bytes']
out.mkdir(exist_ok=False)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def binding(p):return {'path':str(p),'sha256':sha(p),'bytes':p.stat().st_size}
def write(p,d):p.write_text(json.dumps(d,indent=2)+'\n')
folders=['gate99-closure-evidence']
paths=[]
for name in folders:
 folder=root/'benchmarks/runtime/diagnostics'/name
 assert folder.is_dir()
 for p in sorted(folder.rglob('*')):
  if not p.is_file() or p.name=='inventory-plan.json':continue
  assert not p.is_symlink()
  paths.append({'path':p.relative_to(root).as_posix(),'sha256':sha(p),'bytes':p.stat().st_size})
plan={'gate':99,'scope':'Artifact-only exact SQLite import/export of the completed Gate99 allowlisted source, command, output, failure, integration, and acceptance records. No analytics timing samples, source deletion, historical corpus verification, or full-regression claim. Original sources/logs/output/proof bytes remain unchanged.',
 'root':str(root),'database':str(root/'benchmarks/records.sqlite'),'store':binding(root/'benchmarks/storage/store.py'),'analytics':binding(root/'benchmarks/storage/analytics.py'),
 'kind':'artifact','records':paths,'paths':len(paths),'bytes':sum(r['bytes'] for r in paths),'originalsRetained':True}
write(out/'import-plan.json',plan)
print(json.dumps({'plan':binding(out/'import-plan.json'),'paths':plan['paths'],'bytes':plan['bytes']}),flush=True)
spec=importlib.util.spec_from_file_location('store',root/'benchmarks/storage/store.py');store=importlib.util.module_from_spec(spec);spec.loader.exec_module(store)
dbpath=root/'benchmarks/records.sqlite'
def stats():
 db=store.connect(dbpath)
 try:
  return {'analyticsSamples':db.execute('select count(*) from analytics_samples').fetchone()[0],'analyticsProfiles':db.execute('select count(*) from analytics_profiles').fetchone()[0],'records':db.execute('select count(*) from records').fetchone()[0],'blobs':db.execute('select count(*) from blobs').fetchone()[0],'databaseBytes':dbpath.stat().st_size}
 finally:db.close()
before=stats();before['databaseSha256']=sha(dbpath)
start=datetime.datetime.now(datetime.timezone.utc).isoformat();write(out/'start.json',{'argv':['python3','/tmp/tsrust-gate99-archive-plan/import-closed-artifacts.py'],'pid':os.getpid(),'startUTC':start,'before':before,'plan':binding(out/'import-plan.json')})
for r in paths:assert binding(root/r['path'])['sha256']==r['sha256']
count=store.import_records(dbpath,root,[r['path'] for r in paths],kind='artifact')
assert count==len(paths)
scratch=out/'restore';scratch.mkdir()
verified=[]
db=store.connect(dbpath)
try:
 for r in paths:
  # Narrow independent decode of only selected records, then normal no-overwrite export.
  row=db.execute('select records.kind,blobs.sha256,original_size,compressed from records join blobs using(sha256) where records.path=?',(r['path'],)).fetchone()
  assert row is not None and row[0]=='artifact' and row[1]==r['sha256'] and row[2]==r['bytes']
  raw=zlib.decompress(row[3]);assert len(raw)==r['bytes'] and hashlib.sha256(raw).hexdigest()==r['sha256']
  store.export_record(dbpath,r['path'],scratch)
  restored=scratch/r['path'];assert restored.stat().st_size==r['bytes'] and sha(restored)==r['sha256']
  assert sha(root/r['path'])==r['sha256']
  verified.append({'path':r['path'],'sha256':r['sha256'],'bytes':r['bytes'],'compressedBytes':len(row[3]),'kind':'artifact','decodedExact':True,'scratchExportExact':True,'originalRetainedExact':True})
finally:db.close()
for p in [root/'benchmarks/storage/store.py',root/'benchmarks/storage/analytics.py']:
 assert sha(p)==plan['store' if p.name=='store.py' else 'analytics']['sha256']
after=stats();after['databaseSha256']=sha(dbpath);assert after['analyticsSamples']==before['analyticsSamples'] and after['analyticsProfiles']==before['analyticsProfiles']
write(out/'verified-records.json',verified)
result={'gate':99,'accepted':True,'kind':'artifact','analyticsSamplesAdded':0,'importedPaths':count,'narrowVerifiedPaths':len(verified),'originalsRetained':True,'deletedFiles':0,
 'restorationMap':binding(index),'plan':binding(out/'import-plan.json'),'verification':binding(out/'verified-records.json'),'before':before,'after':after,
 'newRecords':after['records']-before['records'],'newBlobs':after['blobs']-before['blobs'],'databaseGrowthBytes':after['databaseBytes']-before['databaseBytes'],
 'selectedOriginalBytes':sum(r['bytes'] for r in paths),'selectedCompressedBytes':sum(r['compressedBytes'] for r in verified),
 'store':plan['store'],'analytics':plan['analytics'],'restoration':'Use store.py export with the recorded relative path and an existing scratch --root. The original absolute path aliases remain in each durable restoration map; no proof bytes were rewritten.'}
write(out/'accepted.json',result)
write(out/'terminal.json',{'argv':['python3','/tmp/tsrust-gate99-archive-plan/import-closed-artifacts.py'],'pid':os.getpid(),'startUTC':start,'finishUTC':datetime.datetime.now(datetime.timezone.utc).isoformat(),'exitCode':0,'accepted':binding(out/'accepted.json')})
print(json.dumps({'accepted':binding(out/'accepted.json'),'terminal':binding(out/'terminal.json'),'importedPaths':count,'before':before,'after':after,'databaseGrowthBytes':result['databaseGrowthBytes'],'selectedOriginalBytes':result['selectedOriginalBytes'],'selectedCompressedBytes':result['selectedCompressedBytes'],'deletedFiles':0}),flush=True)
