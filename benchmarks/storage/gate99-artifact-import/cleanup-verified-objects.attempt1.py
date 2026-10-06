from pathlib import Path
import json,hashlib,os,stat,datetime,sqlite3
ROOT=Path('/Users/seancotter/Desktop/dev/learning/tsrust');D=ROOT/'benchmarks/storage/gate99-artifact-import';acceptedPath=D/'accepted.json';verifiedPath=D/'verified-records.json';a=json.loads(acceptedPath.read_text());v=json.loads(verifiedPath.read_text());MAP=ROOT/'benchmarks/runtime/diagnostics/gate99-closure-evidence/restoration-map.json';SCRATCH=Path('/tmp/tsrust-gate99-bundle-storage/restore');DB=ROOT/'benchmarks/records.sqlite';PREFIX='benchmarks/runtime/diagnostics/gate99-closure-evidence/objects/'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();bind=lambda p:{'path':str(p),'sha256':sha(p),'bytes':p.stat().st_size};now=lambda:datetime.datetime.now(datetime.timezone.utc).isoformat()
assert sha(acceptedPath)=='5776730bd18bdd8b8139c2e865fff0b5554f80ad7490218f4793f4304a3af30f'
assert sha(verifiedPath)==a['verification']['sha256'] and sha(MAP)==a['restorationMap']['sha256']
assert sha(DB)==a['after']['databaseSha256'];selected=[r for r in v if r['path'].startswith(PREFIX)];assert len(selected)==2101
for r in selected:assert r['path']==PREFIX+r['sha256'] and all(r[k] for k in ['decodedExact','scratchExportExact','originalRetainedExact'])
files=[]
def guard(p,r):
 s=p.lstat();assert stat.S_ISREG(s.st_mode) and not p.is_symlink() and s.st_size==r['bytes'];assert sha(p)==r['sha256'];return s
for r in selected:
 for kind,base in [('packagedDuplicate',ROOT),('verifiedScratchDuplicate',SCRATCH)]:
  p=base/r['path'];assert p.parent==base/PREFIX.rstrip('/') and p.parent.resolve()==p.parent
  s=guard(p,r);files.append({'path':str(p),'kind':kind,'sha256':r['sha256'],'bytes':r['bytes'],'allocatedBytes':s.st_blocks*512})
plan={'gate':99,'scope':'Only exact verified packaged object payloads and matching scratch-export payloads; no original aliases, maps, scripts, receipts, sources, workspaces, executables, targets or database records.','importAcceptance':bind(acceptedPath),'verification':bind(verifiedPath),'restorationMap':bind(MAP),'files':files,'count':len(files),'objectCount':len(selected),'logicalBytes':sum(r['bytes'] for r in files),'allocatedBytes':sum(r['allocatedBytes'] for r in files)};(D/'cleanup-plan.json').write_text(json.dumps(plan,indent=2)+'\n')
start=now();deleted=[]
for r in files:
 p=Path(r['path']);guard(p,r);p.unlink();deleted.append(r)
assert all(not Path(r['path']).exists() for r in deleted)
assert sha(MAP)==a['restorationMap']['sha256'] and sha(acceptedPath)=='5776730bd18bdd8b8139c2e865fff0b5554f80ad7490218f4793f4304a3af30f';assert sha(DB)==a['after']['databaseSha256']
c=sqlite3.connect('file:'+str(DB)+'?mode=ro',uri=True)
try:counts={'records':c.execute('select count(*) from records').fetchone()[0],'blobs':c.execute('select count(*) from blobs').fetchone()[0],'analyticsSamples':c.execute('select count(*) from analytics_samples').fetchone()[0],'analyticsProfiles':c.execute('select count(*) from analytics_profiles').fetchone()[0],'databaseBytes':DB.stat().st_size,'databaseSha256':sha(DB)}
finally:c.close()
assert all(counts[k]==a['after'][k] for k in counts)
receipt={'gate':99,'accepted':True,'argv':['python3',str(D/'cleanup-verified-objects.py')],'pid':os.getpid(),'startUTC':start,'finishUTC':now(),'exitCode':0,'plan':bind(D/'cleanup-plan.json'),'importAcceptance':bind(acceptedPath),'restorationMap':bind(MAP),'deletedFiles':len(deleted),'packagedObjectFilesDeleted':len(selected),'scratchObjectFilesDeleted':len(selected),'logicalBytesRemoved':sum(r['bytes'] for r in deleted),'allocatedBytesReclaimed':sum(r['allocatedBytes'] for r in deleted),'databaseGrowthBytes':a['databaseGrowthBytes'],'netAllocatedBytesAfterDatabaseGrowth':sum(r['allocatedBytes'] for r in deleted)-a['databaseGrowthBytes'],'database':counts,'originalSourceAliasesRetained':True,'identityOnlyReferencesRetained':True,'mapsScriptsReceiptsRetained':True,'databaseRecordsDeleted':0,'analyticsSamplesAdded':0,'restoration':'Export the storedPath object using store.py to an existing scratch root, then use restoration-map.json originalPath/digest aliases. Identity-only unsnapshotted versions remain explicitly unavailable.'};(D/'cleanup.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({'cleanup':bind(D/'cleanup.json'),'deletedFiles':receipt['deletedFiles'],'allocatedBytesReclaimed':receipt['allocatedBytesReclaimed'],'netAllocatedBytesAfterDatabaseGrowth':receipt['netAllocatedBytesAfterDatabaseGrowth'],'database':counts}))
