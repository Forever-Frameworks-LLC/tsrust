from pathlib import Path
import datetime,hashlib,json,shutil,sqlite3,subprocess,re
root=Path('/Users/seancotter/Desktop/dev/learning/tsrust');here=Path(__file__).parent;output=root/'benchmarks/storage/closed-gate-73-migration';dbpath=root/'benchmarks/records.sqlite';sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest();load=lambda p:json.loads(Path(p).read_text())
c=load(here/'completed-acceptance.json');r=load(c['runtimeProof']['path']);assert c['gate']==73 and c['compilerAccepted'] and c['runtimeAccepted'] and sha(c['runtimeProof']['path'])==c['runtimeProof']['sha256']
assert load(here/'docs-closure-terminal.json')['exitCode']==0
for key in ['compilerProof','sourceProof']:assert sha(c[key]['path'])==c[key]['sha256']
f=load(here/'runtime-freeze.json')
for path,digest in f['inputs'].items():assert sha(root/path)==digest,path
assert sha(root/'BENCHMARKS.md')==c['report']['sha256']
prior=load(root/'benchmarks/storage/closed-gate-72-migration/postmigration-stats.json');assert sha(dbpath)==prior['databaseSha256'] and dbpath.stat().st_size==prior['databaseBytes']
paths=[]
for proof in [load(c['compilerProof']['path']),load(c['sourceProof']['path']),r]:
 trees={'/'.join(Path(x['path']).parts[1:4]) for x in proof['artifacts']};assert len(trees)==1;paths.extend(trees)
closure=Path(c['runtimeProof']['path']).with_name(Path(c['runtimeProof']['path']).stem+'-closure-artifacts');closure.mkdir()
for p in [here/'completed-acceptance.json',here/'close-docs.py',here/'docs-closure.log',here/'docs-closure-terminal.json']:shutil.copyfile(p,closure/p.name)
paths.append(str(closure.relative_to(root/'benchmarks')))
oldledger=root/'benchmarks/storage/closed-gate-72-migration/protected-files.json';old=load(oldledger)
protected=[root/'BENCHMARKS.md']+[p for folder in ['results','diagnostics'] for p in (root/'benchmarks/runtime'/folder).glob('*.json')]
def identity(p):
 s=p.stat();return [s.st_dev,s.st_ino,s.st_size,s.st_mtime_ns,s.st_ctime_ns]
metadata={str(p.relative_to(root)):identity(p) for p in protected};before={};fresh=[]
for p in protected:
 name=str(p.relative_to(root))
 if name in old and name!='BENCHMARKS.md':assert p.stat().st_size==old[name]['bytes'];before[name]=old[name]
 else:before[name]={'sha256':sha(p),'bytes':p.stat().st_size};fresh.append(name)
assert set(old)<=set(before);assert len(set(before)-set(old))==5
plan={'closedProof':c['runtimeProof'],'paths':paths,'allConsumersFinishedBeforeCompaction':True};planpath=here/'archive-plan.json';planpath.write_text(json.dumps(plan,indent=2)+'\n')
command=['python3',str(root/'benchmarks/storage/compact_closed_gate.py'),'--store',str(root/'benchmarks/storage/store.py'),'--root',str(root/'benchmarks'),'--database',str(dbpath),'--plan',str(planpath),'--output',str(output)]
p=subprocess.run(command,cwd=root,capture_output=True);(here/'storage-utility.stdout').write_bytes(p.stdout);(here/'storage-utility.stderr').write_bytes(p.stderr);assert p.returncode==0,p.stderr.decode()
assert {str(p.relative_to(root)):identity(p) for p in protected}==metadata
for name in fresh:assert sha(root/name)==before[name]['sha256']
for path,digest in f['inputs'].items():assert sha(root/path)==digest
for name,value in [('protected-files.json',before),('protected-verification.json',{'files':len(before),'allPathStatIdentitiesExact':True,'newRecordsAndReportHashesExact':True,'priorLedger':{'path':str(oldledger),'sha256':sha(oldledger)},'priorOriginalBytesRehashed':False})]:(output/name).write_text(json.dumps(value,indent=2)+'\n')
for p in [here/'completed-acceptance.json',here/'close-docs.py',here/'archive.py',here/'storage-utility.stdout',here/'storage-utility.stderr']:shutil.copyfile(p,output/p.name)
m=load(output/'migration.json')
with sqlite3.connect(dbpath.as_uri()+'?mode=ro',uri=True) as db:records=db.execute('SELECT count(*) FROM records').fetchone()[0];blobs=db.execute('SELECT count(*) FROM blobs').fetchone()[0]
cumulative=prior['cumulativeGrossAllocatedBytesReclaimed']+m['grossAllocatedBytesReclaimed'];stats={'gate':73,'records':records,'blobs':blobs,'databaseBytes':dbpath.stat().st_size,'databaseSha256':m['database']['sha256'],'databaseGrowthBytes':m['databaseGrowthBytes'],'files':m['files'],'grossAllocatedBytesReclaimed':m['grossAllocatedBytesReclaimed'],'cumulativeGrossAllocatedBytesReclaimed':cumulative,'protectedFilesExact':len(before),'priorOriginalBytesRehashed':False,'artifactDirectories':paths};(output/'postmigration-stats.json').write_text(json.dumps(stats,indent=2)+'\n')
p=root/'benchmarks/storage/README.md';s=p.read_text();pattern=r'Completed historical/Gate55–72 migrations now store\n\*\*[^*]+\*\*\. Verified\nremoval reclaimed \*\*[^*]+\*\* before migration';replacement=f'Completed historical/Gate55–73 migrations now store\n**{records:,} original records in {blobs:,} blobs, occupying {dbpath.stat().st_size:,} bytes**. Verified\nremoval reclaimed **{cumulative:,} allocated bytes (about {cumulative/1048576:,.0f} MiB)** before migration';s,n=re.subn(pattern,replacement,s);assert n==1;s=s.replace('[Gate72](./closed-gate-72-migration/migration.json). Active gate evidence','[Gate72](./closed-gate-72-migration/migration.json), and\n[Gate73](./closed-gate-73-migration/migration.json). Active gate evidence');p.write_text(s)
p=root/'benchmarks/storage/HISTORICAL_ARCHIVE.md';s=p.read_text();assert '## Closed Gate73 artifacts' not in s;s+=f'\n\n## Closed Gate73 artifacts\n\n[The executed migration](./closed-gate-73-migration/migration.json) archives {m["files"]:,} original integer-formatting evidence records across four finished artifact trees. Independent decoding and scratch restoration verify paths, sizes and SHA256 before removal. All {len(before):,} protected top-level JSON/report files preserve their path and stat identities; new records and the report also preserve fresh hashes. No tests or timings repeat during compaction.\n\nSQLite grows by {m["databaseGrowthBytes"]:,} bytes and reclaims {m["grossAllocatedBytesReclaimed"]:,} allocated bytes gross, before migration-evidence and documentation overhead. The archive now holds {records:,} records in {blobs:,} blobs, using {dbpath.stat().st_size:,} bytes. [Actual statistics](./closed-gate-73-migration/postmigration-stats.json) and the completed acceptance certificate remain durable. Linked raw results remain loose.\n';p.write_text(s);print(json.dumps(stats))
