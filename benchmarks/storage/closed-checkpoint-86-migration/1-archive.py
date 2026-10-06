#!/usr/bin/env python3
"""Root executes once after the completed82–86 checkpoint certificate exists."""
import argparse,hashlib,json,re,shutil,sqlite3,subprocess
from pathlib import Path
ROOT=Path('/Users/seancotter/Desktop/dev/learning/tsrust');BENCH=ROOT/'benchmarks';HERE=Path(__file__).resolve().parent
load=lambda path:json.loads(Path(path).read_bytes())
def sha(path):
    digest=hashlib.sha256()
    with Path(path).open('rb') as stream:
        for block in iter(lambda:stream.read(1048576),b''):digest.update(block)
    return digest.hexdigest()
def bound(item):
    path=Path(item['path']);path=path if path.is_absolute() else ROOT/path
    path=path.resolve(strict=True);assert sha(path)==item['sha256'],str(path);return path

def save(path,value):
    with Path(path).open('x') as stream:json.dump(value,stream,indent=2);stream.write('\n')
def stat_identity(path):
    s=path.stat();return [s.st_dev,s.st_ino,s.st_size,s.st_mtime_ns,s.st_ctime_ns]
def protected_paths():
    return sorted([ROOT/'BENCHMARKS.md']+[p for folder in ['results','diagnostics'] for p in (BENCH/'runtime'/folder).glob('*.json')])
def command_accepted(command):
    assert command['exitCode']==0 and command['argv'] and command['pid']>0
    assert command['startedUtc'] and command['finishedUtc']
    assert sha(command['log'])==command['logSha256']

args=argparse.ArgumentParser(description=__doc__)
args.add_argument('--certificate',required=True);args.add_argument('--certificate-sha256',required=True)
args.add_argument('--output',default=str(BENCH/'storage/closed-checkpoint-86-migration'))
a=args.parse_args();certificate={'path':str(Path(a.certificate).resolve(strict=True)),'sha256':a.certificate_sha256};cp=bound(certificate);c=load(cp)
accepted=['regressionAccepted','runtimeAccepted','profilesAccepted','documentationAccepted','allConsumersFinished']
assert c['gate']==86 and all(c[name] is True for name in accepted)
checkpoint_path=bound(c['checkpointProof']);checkpoint=load(checkpoint_path)
assert checkpoint_path.parent==BENCH/'runtime/diagnostics'
assert checkpoint['gate']==86 and all(checkpoint[name] is True for name in accepted)
assert checkpoint['report']==c['report'] and checkpoint['release']==c['release']
assert bound(c['report'])==ROOT/'BENCHMARKS.md';bound(c['release'])
assert c['commands'] and checkpoint['commands']
for command in c['commands']+checkpoint['commands']:command_accepted(command)
assert checkpoint['disposalAccepted'] is True and isinstance(checkpoint['performance'],dict)
# Explicit completed82–86 proofs plus accepted regression/runtime; no glob selection.
proofs={str(checkpoint_path):c['checkpointProof']};documents={str(checkpoint_path):checkpoint}
for item in c['implementationProofs']:
    path=bound(item);proof=load(path)
    assert path.parent==BENCH/'runtime/diagnostics' and proof['gate'] in range(82,87)
    assert proof.get('implementationAccepted') is True and proof.get('documentationAccepted') is True
    proofs[str(path)]=item;documents[str(path)]=proof
assert {proof['gate'] for proof in documents.values() if proof.get('implementationAccepted')}==set(range(82,87))
for key in ['regressionProof','runtimeProof']:
    item=c.get(key,checkpoint.get(key));assert item,key
    path=bound(item);proof=load(path)
    assert path.parent==BENCH/'runtime/diagnostics' and proof['gate']==86
    assert proof.get('regressionAccepted') is True if key=='regressionProof' else proof.get('runtimeAccepted') is True
    proofs[str(path)]=item;documents[str(path)]=proof
paths=set();artifact_count=0
for proof in documents.values():
    assert proof['artifacts']
    for item in proof['artifacts']:
        path=bound(item);parts=path.relative_to(BENCH).parts
        assert len(parts)>=4 and parts[:2]==('runtime','diagnostics') and parts[2].endswith('-artifacts')
        paths.add('/'.join(parts[:3]));artifact_count+=1
prior_path=BENCH/'storage/closed-checkpoint-81-migration/postmigration-stats.json';prior=load(prior_path)
assert prior['gate']==81 and (prior['records'],prior['blobs'],prior['databaseBytes'])==(18538,12674,126795776)
assert prior['databaseSha256']=='a7f9f07f8bfe89e9ac2d08d0cd721aa97d4285f3924abc185ade15ecb00bf6c2'
dbpath=BENCH/'records.sqlite';assert dbpath.stat().st_size==prior['databaseBytes'] and sha(dbpath)==prior['databaseSha256']
with sqlite3.connect(dbpath.as_uri()+'?mode=ro',uri=True) as db:
    assert db.execute('SELECT count(*) FROM records').fetchone()[0]==prior['records']
    assert db.execute('SELECT count(*) FROM blobs').fetchone()[0]==prior['blobs']
ledger_path=prior_path.with_name('protected-files.json');old=load(ledger_path);assert len(old)==prior['protectedFilesExact']
protected=protected_paths();stats={str(path.relative_to(ROOT)):stat_identity(path) for path in protected};before={};fresh=[]
assert set(old)<=set(stats)
for path in protected:
    name=str(path.relative_to(ROOT))
    if name in old and name!='BENCHMARKS.md':assert path.stat().st_size==old[name]['bytes'];before[name]=old[name]
    else:before[name]={'sha256':sha(path),'bytes':path.stat().st_size};fresh.append(name)
assert all(str(Path(path).relative_to(ROOT)) in before for path in proofs)
# Retain the actual final closure/certificate and command evidence once, without release binaries.
closure=checkpoint_path.with_name(checkpoint_path.stem+'-closure-artifacts');closure.mkdir()
closure_files=[cp,Path(__file__).resolve()]+[Path(command['log']).resolve(strict=True) for command in c['commands']]
closure_index=[]
for index,path in enumerate(dict.fromkeys(closure_files)):
    destination=closure/(str(index)+'-'+path.name);shutil.copyfile(path,destination)
    assert sha(destination)==sha(path);closure_index.append({'originalPath':str(path),'path':str(destination.relative_to(ROOT)),'sha256':sha(destination)})
save(closure/'retained-evidence.json',{'completedCertificate':certificate,'checkpointProof':c['checkpointProof'],'commands':c['commands'],'artifacts':closure_index,'unusedBinariesCopied':False})
paths.add(str(closure.relative_to(BENCH)))
plan={'closedProof':c['checkpointProof'],'completedCertificate':certificate,'completedGateProofs':list(proofs.values()),'paths':sorted(paths),'allConsumersFinishedBeforeCompaction':True}
planpath=HERE/'executed-plan.json';save(planpath,plan)
output=Path(a.output).resolve(strict=False);assert not output.exists()
command=['python3',str(BENCH/'storage/compact_closed_gate.py'),'--store',str(BENCH/'storage/store.py'),'--root',str(BENCH),'--database',str(dbpath),'--plan',str(planpath),'--output',str(output)]
result=subprocess.run(command,cwd=ROOT,capture_output=True)
(HERE/'utility.stdout').write_bytes(result.stdout);(HERE/'utility.stderr').write_bytes(result.stderr)
assert result.returncode==0,result.stderr.decode()
assert {str(path.relative_to(ROOT)):stat_identity(path) for path in protected_paths()}==stats
for name in fresh:assert sha(ROOT/name)==before[name]['sha256']
assert sha(ROOT/'BENCHMARKS.md')==c['report']['sha256']
save(output/'protected-files.json',before)
save(output/'protected-verification.json',{'files':len(before),'allPathStatIdentitiesExact':True,'newRecordsAndReportHashesExact':True,'freshHashes':fresh,'inheritedOriginalHashes':sorted(set(old)-{'BENCHMARKS.md'}),'priorLedger':{'path':str(ledger_path),'sha256':sha(ledger_path)},'priorOriginalBytesRehashed':False})
for index,path in enumerate([cp,Path(__file__).resolve(),planpath,HERE/'utility.stdout',HERE/'utility.stderr']):shutil.copyfile(path,output/(str(index)+'-'+path.name))
m=load(output/'migration.json')
with sqlite3.connect(dbpath.as_uri()+'?mode=ro',uri=True) as db:
    records=db.execute('SELECT count(*) FROM records').fetchone()[0];blobs=db.execute('SELECT count(*) FROM blobs').fetchone()[0]
cumulative=prior['cumulativeGrossAllocatedBytesReclaimed']+m['grossAllocatedBytesReclaimed']
actual={'gate':86,'completedGates':[82,83,84,85,86],'records':records,'blobs':blobs,'databaseBytes':dbpath.stat().st_size,'databaseSha256':m['database']['sha256'],'databaseGrowthBytes':m['databaseGrowthBytes'],'files':m['files'],'originalLogicalBytes':m['originalLogicalBytes'],'originalAllocatedBytes':m['originalAllocatedBytes'],'grossAllocatedBytesReclaimed':m['grossAllocatedBytesReclaimed'],'cumulativeGrossAllocatedBytesReclaimed':cumulative,'protectedFilesExact':len(before),'priorOriginalBytesRehashed':False,'artifactDirectories':sorted(paths),'completedCertificate':certificate,'checkpointProof':c['checkpointProof']}
save(output/'postmigration-stats.json',actual)
# Guides change only after actual successful compaction. All top-level report/raw links stay loose; this artifact-only owner never selects them.
p=BENCH/'storage/README.md';text=p.read_text();pattern=r'Completed historical/Gate55–81 migrations now store\n\*\*[^*]+\*\*\. Verified\nremoval reclaimed \*\*[^*]+\*\* before migration'
replacement=f'Completed historical/Gate55–86 migrations now store\n**{records:,} original records in {blobs:,} blobs, occupying {dbpath.stat().st_size:,} bytes**. Verified\nremoval reclaimed **{cumulative:,} allocated bytes (about {cumulative/1048576:,.0f} MiB)** before migration'
text,count=re.subn(pattern,replacement,text);assert count==1
oldlink='[checkpoint77–81](./closed-checkpoint-81-migration/migration.json).';assert oldlink in text
text=text.replace(oldlink,'[checkpoint77–81](./closed-checkpoint-81-migration/migration.json), and\n[checkpoint82–86](./closed-checkpoint-86-migration/migration.json).',1);p.write_text(text)
p=BENCH/'storage/HISTORICAL_ARCHIVE.md';text=p.read_text();assert '## Closed checkpoint82–86 artifacts' not in text
text+=f'\n\n## Closed checkpoint82–86 artifacts\n\n[The executed migration](./closed-checkpoint-86-migration/migration.json) archives {m["files"]:,} original records from {len(paths)} completed proof-derived artifact trees, including accepted implementation, regression, runtime and checkpoint closure evidence. Every selected original path, size and SHA256 matches independently decoded archive bytes and scratch restoration before removal; each leaf is rechecked immediately before unlink. All {len(before):,} loose top-level JSON/report paths and stat identities remain exact. New records and the report retain fresh hashes; historical hashes inherit the bound checkpoint81 ledger without rehashing earlier raw records. No tests, builds or timings repeat during compaction. The unchanged compactor also validates the stored database using its existing verification routine.\n\nSQLite grows by {m["databaseGrowthBytes"]:,} bytes, reclaiming {m["grossAllocatedBytesReclaimed"]:,} allocated bytes gross before migration evidence and documentation overhead. The archive now holds {records:,} records in {blobs:,} blobs, using {dbpath.stat().st_size:,} bytes. [Actual statistics](./closed-checkpoint-86-migration/postmigration-stats.json) bind the database SHA256 and cumulative gross savings of {cumulative:,} bytes. Linked raw results, profiles, top-level proofs and BENCHMARKS.md remain loose.\n';p.write_text(text)
print(json.dumps(actual))
