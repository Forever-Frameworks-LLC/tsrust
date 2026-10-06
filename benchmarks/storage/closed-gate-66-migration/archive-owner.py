"""Compact only the four actual closed Gate66 evidence trees after all consumers finish."""
from pathlib import Path
import argparse, datetime, hashlib, json, os, re, shutil, sqlite3, subprocess, sys

ROOT = Path('/Users/seancotter/Desktop/dev/learning/tsrust')
OUTPUT = ROOT/'benchmarks/storage/closed-gate-66-migration'
DATABASE = ROOT/'benchmarks/records.sqlite'
parser = argparse.ArgumentParser(description=__doc__)
for name in ['certificate', 'certificate-sha', 'runtime-proof', 'runtime-proof-sha', 'readme-supplement', 'readme-supplement-sha']:
    parser.add_argument('--'+name, required=True)
args = parser.parse_args()
def sha(path):
    digest = hashlib.sha256()
    with Path(path).open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024*1024), b''): digest.update(chunk)
    return digest.hexdigest()
def load(path): return json.loads(Path(path).read_text())
def save(path, value):
    with Path(path).open('x') as target: json.dump(value, target, indent=2); target.write('\n')
def bound(path, expected):
    path = Path(path).resolve(strict=True)
    assert sha(path) == expected, str(path)
    return path

def snapshot():
    files = [ROOT/'BENCHMARKS.md']+[file for folder in [ROOT/'benchmarks/runtime/results', ROOT/'benchmarks/runtime/diagnostics'] for file in folder.glob('*.json')]
    return {str(file.relative_to(ROOT)):{'sha256':sha(file),'bytes':file.stat().st_size} for file in files}

def artifact_tree(artifacts):
    # Every identity comes from the accepted proof, never from an anticipated date.
    assert artifacts
    trees = set()
    for artifact in artifacts:
        parts = Path(artifact['path']).parts
        assert len(parts) >= 5 and parts[:3] == ('benchmarks','runtime','diagnostics')
        assert parts[3].endswith('-artifacts')
        trees.add('/'.join(parts[1:4]))
    assert len(trees) == 1
    return trees.pop()

certificate = bound(args.certificate,args.certificate_sha)
runtime_path = bound(args.runtime_proof,args.runtime_proof_sha)
supplement = bound(args.readme_supplement,args.readme_supplement_sha)
closed,runtime,docs = load(certificate),load(runtime_path),load(supplement)
runtime_binding = {'path':str(runtime_path),'sha256':args.runtime_proof_sha}
assert closed['gate'] == docs['gate'] == runtime['gate'] == 66
assert closed['compilerAccepted'] and closed['runtimeAccepted']
assert closed['runtimeProof'] == docs['runtimeProof'] == runtime_binding
assert docs['measurementsRepeated'] is False
assert closed['documentationSupplementSha256'] == args.readme_supplement_sha
assert Path(closed['documentationOnlyCompilerSupplement']).resolve() == supplement
assert runtime['scope']['allAfterOutputAndFinalDisposalsZero']
assert closed['measuredRows'] == runtime['scope']['groupedReportMeasuredRows']
proofs = []
for name in ['compilerProof','sourceProof']:
    identity = closed[name]
    proof = load(bound(identity['path'],identity['sha256']))
    assert proof['gate'] == 66 and docs[name] == identity
    proofs.append(proof)
paths = [artifact_tree(proof['artifacts']) for proof in [*proofs,runtime]]
closure = closed['closureArtifacts']+[closed['archivedDocumentationSupplement']]
paths.append(artifact_tree(closure))
assert len(paths) == len(set(paths)) == 4
assert not OUTPUT.exists()
for relative in paths: assert (ROOT/'benchmarks'/relative).is_dir(), relative
plan = {'closedProof':runtime_binding,'paths':paths,
        'completedAcceptance':{'path':str(certificate),'sha256':args.certificate_sha},
        'allConsumersFinishedBeforeCompaction':True}
work = Path('/tmp/tsrust-gate66-archive-execution')
work.mkdir(exist_ok=False)
plan_path = work/'plan.json'; save(plan_path,plan)
before = snapshot(); save(work/'protected-before.json',before)
command = [sys.executable,str(ROOT/'benchmarks/storage/compact_closed_gate.py'),'--store',str(ROOT/'benchmarks/storage/store.py'),'--root',str(ROOT/'benchmarks'),'--database',str(DATABASE),'--plan',str(plan_path),'--output',str(OUTPUT)]
started = datetime.datetime.now(datetime.timezone.utc).isoformat()
log_path = work/'utility.log'
with log_path.open('xb') as log:
    process = subprocess.Popen(command,cwd=ROOT,stdout=log,stderr=subprocess.STDOUT)
    code = process.wait()
record = {'gate':66,'argv':command,'cwd':str(ROOT),'pid':process.pid,'startedUtc':started,
          'finishedUtc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'exitCode':code,
          'log':str(log_path),'logSha256':sha(log_path)}
save(work/'utility-terminal.json',record)
assert code == 0, 'Compaction failed; actual log and terminal retained in '+str(work)
after = snapshot(); assert after == before, 'Protected loose original paths or bytes changed'
save(OUTPUT/'protected-files.json',before)
save(OUTPUT/'protected-verification.json',{'files':len(before),'completePathSetAndHashesExact':True})
for source,name in [(certificate,'completed-acceptance.json'),(supplement,'postclosure-readme-supplement.json'),
                    (Path(__file__).resolve(),'archive-owner.py'),(log_path,'utility.log'),
                    (work/'utility-terminal.json','utility-terminal.json')]:
    shutil.copyfile(source,OUTPUT/name)
migration = load(OUTPUT/'migration.json')
assert migration['selection'] == paths and migration['verification']
with sqlite3.connect(DATABASE.as_uri()+'?mode=ro',uri=True) as db:
    records = db.execute('SELECT count(*) FROM records').fetchone()[0]
    blobs = db.execute('SELECT count(*) FROM blobs').fetchone()[0]
# Actual migration manifests supply historical savings; no archive corpus scan.
historical = load(ROOT/'benchmarks/storage/historical-artifact-migration-2026-10-02.json')
cumulative = historical['savings']['allocatedBytesBeforeDocumentation'] + sum(load(path)['grossAllocatedBytesReclaimed'] for path in (ROOT/'benchmarks/storage').glob('closed-gate-*-migration/migration.json'))
stats_path = OUTPUT/'postmigration-stats.json'; stats_path.write_text('{}\n')
guide_record = OUTPUT/'storage-guide-supplement.json'; guide_record.write_text('{}\n')
evidence_bytes = sum(file.stat().st_blocks*512 for file in OUTPUT.iterdir() if file.is_file())
stats = {'gate':66,'records':records,'blobs':blobs,'databaseBytes':DATABASE.stat().st_size,
         'databaseSha256':migration['database']['sha256'],'databaseGrowthBytes':migration['databaseGrowthBytes'],
         'files':migration['files'],'artifactDirectories':paths,'originalLogicalBytes':migration['originalLogicalBytes'],
         'originalAllocatedBytes':migration['originalAllocatedBytes'],'grossAllocatedBytesReclaimed':migration['grossAllocatedBytesReclaimed'],
         'migrationEvidenceAllocatedBytes':evidence_bytes,'netAllocatedBytesReclaimedAfterEvidence':migration['grossAllocatedBytesReclaimed']-evidence_bytes,
         'cumulativeGrossAllocatedBytesReclaimed':cumulative,'protectedFilesExact':len(before)}
stats_path.write_text(json.dumps(stats,indent=2)+'\n')
readme = ROOT/'benchmarks/storage/README.md'; history = ROOT/'benchmarks/storage/HISTORICAL_ARCHIVE.md'
guides_before = {str(path.relative_to(ROOT)):sha(path) for path in [readme,history]}
text = readme.read_text()
pattern = r'Completed historical/Gate55–65 migrations now store\n\*\*[^*]+\*\*\. Verified\nremoval reclaimed \*\*[^*]+\*\* before migration'
replacement = f'Completed historical/Gate55–66 migrations now store\n**{records:,} original records in {blobs:,} blobs, occupying {DATABASE.stat().st_size:,} bytes**. Verified\nremoval reclaimed **{cumulative:,} allocated bytes (about {cumulative/1048576:,.0f} MiB)** before migration'
text,count = re.subn(pattern,replacement,text); assert count == 1
old = '[Gate65](./closed-gate-65-migration/migration.json). Active gate evidence'
assert text.count(old) == 1
text = text.replace(old,'[Gate65](./closed-gate-65-migration/migration.json), and\n[Gate66](./closed-gate-66-migration/migration.json). Active gate evidence')
readme.write_text(text)
text = history.read_text(); assert '## Closed Gate66 artifacts' not in text
text += f'''\n\n## Closed Gate66 artifacts\n\nGate66's compiler, source, runtime and closure artifact directories are archived\nunder [the exact completed-gate plan](./closed-gate-66-migration/plan.json).\n[The executed migration](./closed-gate-66-migration/migration.json) preserves\n{migration['files']:,} original records, including the failed full regression, failed first\nparser repair and measured frame diagnosis. Independent decoding and scratch\nrestoration verified original paths, sizes and SHA256 before removal. All\n{len(before):,} protected top-level JSON/report paths and bytes remain unchanged.\nThe completed certificate and documentation supplement are retained with the\nmigration evidence. No tests or benchmark measurements repeated during compaction.\n\nOriginal artifacts occupied {migration['originalLogicalBytes']:,} logical bytes and\n{migration['originalAllocatedBytes']:,} allocated bytes. SQLite grew by\n{migration['databaseGrowthBytes']:,} bytes, reclaiming {migration['grossAllocatedBytesReclaimed']:,}\nallocated bytes gross and {stats['netAllocatedBytesReclaimedAfterEvidence']:,} bytes after\nmigration evidence, before guide growth. Restore only the four exact prefixes\nin the saved plan when historical consumers need original files.\n\nCurrent archive: **{records:,} records / {blobs:,} unique blobs / {DATABASE.stat().st_size:,} bytes**\n(about {DATABASE.stat().st_size/1048576:,.1f} MiB). Combined gross allocated savings total\n**{cumulative:,} bytes (about {cumulative/1048576:,.0f} MiB)** before evidence and\ndocumentation overhead. [The actual inventory](./closed-gate-66-migration/postmigration-stats.json)\nretains exact counts and sizes. Database SHA256 at this migration:\n`{migration['database']['sha256']}`. Linked raw results and top-level proofs remain loose.\n'''
history.write_text(text)
guide_record.write_text(json.dumps({'gate':66,'kind':'documentation-only-postcompaction-storage-guides',
    'utilityExitCode':code,'executedMigration':{'path':str(OUTPUT/'migration.json'),'sha256':sha(OUTPUT/'migration.json')},
    'files':[{'path':relative,'beforeSha256':digest,'afterSha256':sha(ROOT/relative)} for relative,digest in guides_before.items()],
    'testsRepeated':False,'measurementsRepeated':False},indent=2)+'\n')
assert evidence_bytes == sum(file.stat().st_blocks*512 for file in OUTPUT.iterdir() if file.is_file())
print(json.dumps(stats))
