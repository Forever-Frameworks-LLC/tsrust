# Source-only proposal. Root executes after certificate and every actual consumer completes.
from pathlib import Path
import hashlib,json,shutil,sqlite3,subprocess,re
root=Path('/Users/seancotter/Desktop/dev/learning/tsrust');here=Path(__file__).resolve().parent
output=root/'benchmarks/storage/closed-gate-76-migration';dbpath=root/'benchmarks/records.sqlite'
load=lambda path:json.loads(Path(path).read_text())
def sha(path):
 digest=hashlib.sha256()
 with Path(path).open('rb') as stream:
  for chunk in iter(lambda:stream.read(1048576),b''):digest.update(chunk)
 return digest.hexdigest()
def bound(item):
 path=Path(item['path']).resolve(strict=True);assert sha(path)==item['sha256'],str(path);return path
c=load(here/'completed-acceptance.json');assert c['gate']==76 and c['compilerAccepted'] and c['runtimeAccepted'] and c['measuredRows']==488
cp,sp,rp=[bound(c[key]) for key in ['compilerProof','sourceProof','runtimeProof']];compiler,source,r=[load(path) for path in [cp,sp,rp]]
assert compiler['gate']==source['gate']==r['gate']==76 and compiler['compilerAccepted'] and source['sourceAccepted'] and r['runtimeAccepted'] and r['timed']
assert r['compilerProof']==c['compilerProof'] and r['sourceProof']==c['sourceProof']
assert r['scope']['old484MeasurementTablesByteExact'] and r['scope']['old9MemoryTablesByteExact'] and r['scope']['auxiliaryTablesByteExact']
assert r['scope']['allDisposalsZero'] and r['scope']['newWorkloads']==4 and r['scope']['groupedReportMeasuredRows']==488
assert r['scope']['processCells']==96 and r['scope']['timedBatches']==2976 and r['scope']['nativeProfileSamples']==12
assert sha(Path(c['releasePath']))==c['releaseSha256']==r['acceptedReleaseBinarySha256']
assert compiler['regression']['accepted'] and compiler['regression']['fullRestarts']==c['fullRestarts']==0
assert compiler['regression']['uniqueGroups']==c['namedGroups']==661 and compiler['regression']['totalWorkspaceGroups']==c['workspaceNamedGroups']==680
assert all(item['exitCode']==0 and sha(item['log'])==item['logSha256'] for item in compiler['finalChecks']+source['commands'])
assert all(item['completeOutputAccepted'] for item in compiler['nativePoolAndOrder']['fixtures'])
for name in ['full','native-hands','runtime','profiles','report','docs-closure']:
 terminal=load(here/(name+'-terminal.json'));assert terminal['gate']==76 and terminal['exitCode']==0 and sha(terminal['log'])==terminal['logSha256'],name
 if name in r['sessions']:assert terminal=={key:value for key,value in r['sessions'][name].items() if key not in ['path','sha256']}
f=load(here/'runtime-freeze.json');assert f['gate']==76 and f['compilerProof']==c['compilerProof'] and f['sourceProof']==c['sourceProof']
full=load(here/'full-source-freeze.json');assert full['gate']==76 and len(full['sourceIdentities'])==145
assert compiler['currentSourceIdentities']==full['sourceIdentities']
assert all(f['inputs'].get(path)==digest for path,digest in full['sourceIdentities'].items())
for path,digest in f['inputs'].items():assert sha(root/path)==digest,path
assert sha(root/'BENCHMARKS.md')==c['report']['sha256']==r['reportSha256']
priorpath=root/'benchmarks/storage/closed-gate-75-migration/postmigration-stats.json';prior=load(priorpath)
assert prior['gate']==75 and prior['records']==17580 and prior['blobs']==11876 and prior['databaseBytes']==123412480 and prior['databaseSha256']=='bed159ae74427da359416ddb1b4433083355b15c142e499c6be4e14ba62fb099' and prior['cumulativeGrossAllocatedBytesReclaimed']==1516593152
assert sha(dbpath)==prior['databaseSha256'] and dbpath.stat().st_size==prior['databaseBytes']
with sqlite3.connect(dbpath.as_uri()+'?mode=ro',uri=True) as db:assert db.execute('SELECT count(*) FROM records').fetchone()[0]==prior['records'] and db.execute('SELECT count(*) FROM blobs').fetchone()[0]==prior['blobs']
paths=[]
for proof in [compiler,source,r]:
 trees={'/'.join(Path(item['path']).parts[1:4]) for item in proof['artifacts']};assert len(trees)==1
 tree=next(iter(trees));assert tree.startswith('runtime/diagnostics/') and 'string-well-formed-' in tree and tree.endswith('-artifacts');paths.append(tree)
closure=rp.with_name(rp.stem+'-closure-artifacts');assert closure.parent==root/'benchmarks/runtime/diagnostics';closure.mkdir()
closure_files=[here/name for name in ['completed-acceptance.json','close-docs.py','docs-closure-start.json','docs-closure.log','docs-closure-terminal.json','archive.py']]
for path in closure_files:shutil.copyfile(path,closure/path.name);assert sha(path)==sha(closure/path.name)
paths.append(str(closure.relative_to(root/'benchmarks')));assert len(paths)==len(set(paths))==4
oldledger=root/'benchmarks/storage/closed-gate-75-migration/protected-files.json';old=load(oldledger);assert len(old)==prior['protectedFilesExact']==500
def protected_paths():return [root/'BENCHMARKS.md']+[path for folder in ['results','diagnostics'] for path in (root/'benchmarks/runtime'/folder).glob('*.json')]
def identity(path):
 s=path.stat();return [s.st_dev,s.st_ino,s.st_size,s.st_mtime_ns,s.st_ctime_ns]
protected=protected_paths();metadata={str(path.relative_to(root)):identity(path) for path in protected};before={};fresh=[]
for path in protected:
 name=str(path.relative_to(root))
 if name in old and name!='BENCHMARKS.md':assert path.stat().st_size==old[name]['bytes'];before[name]=old[name]
 else:before[name]={'sha256':sha(path),'bytes':path.stat().st_size};fresh.append(name)
new_loose={str(path.relative_to(root)) for path in [cp,sp,rp]+[bound(item) for item in r['records']]};assert len(new_loose)==5 and set(before)-set(old)==new_loose and set(old)<=set(before)
plan={'closedProof':c['runtimeProof'],'completedCertificate':{'path':str(here/'completed-acceptance.json'),'sha256':sha(here/'completed-acceptance.json')},'paths':paths,'allConsumersFinishedBeforeCompaction':True};planpath=here/'archive-plan.json'
with planpath.open('x') as stream:json.dump(plan,stream,indent=2);stream.write('\n')
command=['python3',str(root/'benchmarks/storage/compact_closed_gate.py'),'--store',str(root/'benchmarks/storage/store.py'),'--root',str(root/'benchmarks'),'--database',str(dbpath),'--plan',str(planpath),'--output',str(output)]
result=subprocess.run(command,cwd=root,capture_output=True);(here/'storage-utility.stdout').write_bytes(result.stdout);(here/'storage-utility.stderr').write_bytes(result.stderr);assert result.returncode==0,result.stderr.decode()
assert {str(path.relative_to(root)):identity(path) for path in protected_paths()}==metadata
for name in fresh:assert sha(root/name)==before[name]['sha256']
for path,digest in f['inputs'].items():assert sha(root/path)==digest,path
for name,value in [('protected-files.json',before),('protected-verification.json',{'files':len(before),'allPathStatIdentitiesExact':True,'newRecordsAndReportHashesExact':True,'freshHashes':fresh,'inheritedOriginalHashes':sorted(set(old)-{'BENCHMARKS.md'}),'priorLedger':{'path':str(oldledger),'sha256':sha(oldledger)},'priorOriginalBytesRehashed':False})]:
 with (output/name).open('x') as stream:json.dump(value,stream,indent=2);stream.write('\n')
for path in closure_files+[here/'storage-utility.stdout',here/'storage-utility.stderr',here/'archive-plan.json']:shutil.copyfile(path,output/path.name)
m=load(output/'migration.json')
with sqlite3.connect(dbpath.as_uri()+'?mode=ro',uri=True) as db:records=db.execute('SELECT count(*) FROM records').fetchone()[0];blobs=db.execute('SELECT count(*) FROM blobs').fetchone()[0]
cumulative=prior['cumulativeGrossAllocatedBytesReclaimed']+m['grossAllocatedBytesReclaimed'];stats={'gate':76,'records':records,'blobs':blobs,'databaseBytes':dbpath.stat().st_size,'databaseSha256':m['database']['sha256'],'databaseGrowthBytes':m['databaseGrowthBytes'],'files':m['files'],'originalLogicalBytes':m['originalLogicalBytes'],'originalAllocatedBytes':m['originalAllocatedBytes'],'grossAllocatedBytesReclaimed':m['grossAllocatedBytesReclaimed'],'cumulativeGrossAllocatedBytesReclaimed':cumulative,'protectedFilesExact':len(before),'priorOriginalBytesRehashed':False,'artifactDirectories':paths}
with (output/'postmigration-stats.json').open('x') as stream:json.dump(stats,stream,indent=2);stream.write('\n')
path=root/'benchmarks/storage/README.md';text=path.read_text();pattern=r'Completed historical/Gate55–75 migrations now store\n\*\*[^*]+\*\*\. Verified\nremoval reclaimed \*\*[^*]+\*\* before migration';replacement=f'Completed historical/Gate55–76 migrations now store\n**{records:,} original records in {blobs:,} blobs, occupying {dbpath.stat().st_size:,} bytes**. Verified\nremoval reclaimed **{cumulative:,} allocated bytes (about {cumulative/1048576:,.0f} MiB)** before migration';text,n=re.subn(pattern,replacement,text);assert n==1;text=text.replace('[Gate75](./closed-gate-75-migration/migration.json). Active gate evidence','[Gate75](./closed-gate-75-migration/migration.json), and\n[Gate76](./closed-gate-76-migration/migration.json). Active gate evidence');assert '[Gate76]' in text;path.write_text(text)
path=root/'benchmarks/storage/HISTORICAL_ARCHIVE.md';text=path.read_text();assert '## Closed Gate76 artifacts' not in text;text+=f'\n\n## Closed Gate76 artifacts\n\n[The executed migration](./closed-gate-76-migration/migration.json) archives {m["files"]:,} original String.isWellFormed/String.toWellFormed evidence records across four finished artifact trees. Independent decoding and scratch restoration verify every original path, size and SHA256 before removal, with each leaf rechecked immediately before unlink. All {len(before):,} protected top-level JSON/report paths and stat identities remain exact; new records and the report also preserve fresh hashes. Earlier record hashes are inherited from the bound Gate75 ledger without rehashing that corpus. No tests or timings repeat during compaction.\n\nSQLite grows by {m["databaseGrowthBytes"]:,} bytes and reclaims {m["grossAllocatedBytesReclaimed"]:,} allocated bytes gross, before migration-evidence and documentation overhead. The archive now holds {records:,} records in {blobs:,} blobs, using {dbpath.stat().st_size:,} bytes. [Actual statistics](./closed-gate-76-migration/postmigration-stats.json) bind the database SHA256 and cumulative gross savings of {cumulative:,} bytes. The completed acceptance certificate, unchanged observer bindings and original command evidence remain durable. Linked raw results and top-level proofs remain loose.\n';path.write_text(text);print(json.dumps(stats))
