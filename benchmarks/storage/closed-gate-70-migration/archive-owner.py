"""Compact only the four actual closed Gate70 evidence trees after all consumers finish."""
from pathlib import Path
import argparse, datetime, hashlib, json, os, re, shutil, sqlite3, subprocess, sys

ROOT = Path('/Users/seancotter/Desktop/dev/learning/tsrust')
OUTPUT = ROOT/'benchmarks/storage/closed-gate-70-migration'
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

PRIOR_PROTECTED = ROOT/'benchmarks/storage/closed-gate-69-migration/protected-files.json'
PRIOR_LEDGER_SHA = 'd3e835bafbfab109d4309c22065cd966ac4f0810896134f0c7027e1dbae0a71c'
def protected_files():
    return [ROOT/'BENCHMARKS.md']+[file for folder in [ROOT/'benchmarks/runtime/results', ROOT/'benchmarks/runtime/diagnostics'] for file in folder.glob('*.json')]
def stat_identity(file):
    assert file.is_file() and not file.is_symlink()
    value = file.stat()
    return {'device':value.st_dev,'inode':value.st_ino,'bytes':value.st_size,'mtimeNs':value.st_mtime_ns,'ctimeNs':value.st_ctime_ns}
def snapshot_before():
    assert sha(PRIOR_PROTECTED) == PRIOR_LEDGER_SHA
    prior = load(PRIOR_PROTECTED); assert len(prior) == 466
    identities, metadata, freshly_hashed = {}, {}, []
    for file in protected_files():
        relative = str(file.relative_to(ROOT)); metadata[relative] = stat_identity(file)
        if relative in prior and relative != 'BENCHMARKS.md':
            assert file.stat().st_size == prior[relative]['bytes']; identities[relative] = prior[relative]
        else:
            identities[relative] = {'sha256':sha(file),'bytes':file.stat().st_size}; freshly_hashed.append(relative)
    assert set(prior) <= set(identities)
    assert set(identities)-set(prior) == new_loose_records
    assert set(freshly_hashed) == new_loose_records | {'BENCHMARKS.md'}
    return identities,metadata,freshly_hashed

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
assert closed['gate'] == docs['gate'] == runtime['gate'] == 70
assert closed['compilerAccepted'] and closed['runtimeAccepted']
assert closed['runtimeProof'] == docs['runtimeProof'] == runtime_binding
assert docs['measurementsRepeated'] is False
assert closed['documentationSupplementSha256'] == args.readme_supplement_sha
assert Path(closed['documentationOnlyCompilerSupplement']).resolve() == supplement
assert runtime['scope']['allAfterOutputAndFinalDisposalsZero']
assert closed['measuredRows'] == runtime['scope']['groupedReportMeasuredRows'] == 454
assert runtime['timed'] is True
assert runtime['scope']['newWorkloads'] == 10 and runtime['scope']['old444MeasurementTablesByteExact'] and runtime['scope']['old444ReportSectionsByteExact']
assert runtime['scope']['old9PrimitiveMemoryTablesByteExact'] and runtime['scope']['auxiliaryTablesByteExact']
proofs = []
for name in ['compilerProof','sourceProof']:
    identity = closed[name]; assert runtime[name] == identity
    proof = load(bound(identity['path'],identity['sha256']))
    assert proof['gate'] == 70 and docs[name] == identity
    proofs.append(proof)
paths = [artifact_tree(proof['artifacts']) for proof in [*proofs,runtime]]
closure = closed['closureArtifacts']+[closed['archivedDocumentationSupplement']]
paths.append(artifact_tree(closure))
assert runtime_path.name.endswith('-string-concat-padding-runtime-proof.json')
runtime_stem = runtime_path.name.removesuffix('-proof.json')
expected_paths = [
 'runtime/diagnostics/2026-10-03T00-07-33-000Z-string-concat-padding-compiler-proof-artifacts',
 'runtime/diagnostics/2026-10-02T23-57-28-000Z-string-concat-padding-source-validation-artifacts',
 'runtime/diagnostics/'+runtime_stem+'-artifacts',
 'runtime/diagnostics/'+runtime_stem+'-proof-closure-artifacts']
assert paths == expected_paths and len(paths) == len(set(paths)) == 4
closure_terminal = Path('/tmp/tsrust-gate70-root-acceptance/docs-closure-terminal.json').resolve(strict=True)
closure_actual = load(closure_terminal)
assert closure_actual['exitCode'] == 0 and sha(closure_actual['log']) == closure_actual['logSha256']
assert sha(DATABASE) == '54398d8a5f853649e1c17a8c2572afadb26addd050cd33edb8e4511322d4119f'
with sqlite3.connect(DATABASE.as_uri()+'?mode=ro',uri=True) as db:
    assert (db.execute('SELECT count(*) FROM records').fetchone()[0],db.execute('SELECT count(*) FROM blobs').fetchone()[0],DATABASE.stat().st_size) == (15065,10353,115400704)
# Derive the new loose set from actual accepted public bindings, never planned counts.
prior = load(PRIOR_PROTECTED)
public_bindings = {str(runtime_path.relative_to(ROOT)):args.runtime_proof_sha}
for identity in [*runtime['records'],closed['compilerProof'],closed['sourceProof']]:
    file = Path(identity['path'])
    if not file.is_absolute(): file = ROOT/file
    file = file.resolve(strict=True)
    assert file.parent in [ROOT/'benchmarks/runtime/results',ROOT/'benchmarks/runtime/diagnostics'] and file.suffix == '.json'
    relative = str(file.relative_to(ROOT))
    assert relative not in public_bindings or public_bindings[relative] == identity['sha256']
    public_bindings[relative] = identity['sha256']
new_loose_records = set(public_bindings)-set(prior)
assert new_loose_records
for relative in new_loose_records: bound(ROOT/relative,public_bindings[relative])
# Narrow current production sources and support; no historical input scan.
source_bindings = {item['path']:item['sha256'] for item in proofs[1]['currentSourceFiles']}
for name,digest in proofs[1]['unchangedBenchmarkSupport'].items():
    relative = 'benchmarks/runtime/'+name
    assert relative not in source_bindings or source_bindings[relative] == digest
    source_bindings[relative] = digest
for relative,digest in source_bindings.items(): bound(ROOT/relative,digest)
repair_binding = runtime['sourcePackagingRepair']
repair_path = bound(repair_binding['path'],repair_binding['sha256'])
assert repair_path.is_relative_to(ROOT/'benchmarks'/paths[1])
repair = load(repair_path)
assert repair['gate'] == 70 and repair['sourceProof'] == closed['sourceProof']
assert repair['packagingFailedOnce'] is True and repair['sourceValidationRepeated'] is False and repair['observationsMovedOnce'] is True
for identity in repair['records']:
    assert bound(identity['path'],identity['sha256']).is_relative_to(ROOT/'benchmarks'/paths[1])
assert not OUTPUT.exists()
for relative in paths: assert (ROOT/'benchmarks'/relative).is_dir(), relative
plan = {'closedProof':runtime_binding,'paths':paths,
        'completedAcceptance':{'path':str(certificate),'sha256':args.certificate_sha},
        'allConsumersFinishedBeforeCompaction':True,'sourcePackagingRepair':repair_binding}
work = Path('/tmp/tsrust-gate70-archive-execution')
work.mkdir(exist_ok=False)
plan_path = work/'plan.json'; save(plan_path,plan)
before, metadata, freshly_hashed = snapshot_before()
assert all(before[relative]['sha256'] == public_bindings[relative] for relative in new_loose_records)
save(work/'protected-before.json',before); save(work/'protected-metadata-before.json',metadata)
command = [sys.executable,str(ROOT/'benchmarks/storage/compact_closed_gate.py'),'--store',str(ROOT/'benchmarks/storage/store.py'),'--root',str(ROOT/'benchmarks'),'--database',str(DATABASE),'--plan',str(plan_path),'--output',str(OUTPUT)]
started = datetime.datetime.now(datetime.timezone.utc).isoformat()
log_path = work/'utility.log'
with log_path.open('xb') as log:
    process = subprocess.Popen(command,cwd=ROOT,stdout=log,stderr=subprocess.STDOUT)
    code = process.wait()
record = {'gate':70,'argv':command,'cwd':str(ROOT),'pid':process.pid,'startedUtc':started,
          'finishedUtc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'exitCode':code,
          'log':str(log_path),'logSha256':sha(log_path)}
save(work/'utility-terminal.json',record)
assert code == 0, 'Compaction failed; actual log and terminal retained in '+str(work)
after_files = {str(file.relative_to(ROOT)):file for file in protected_files()}
assert set(after_files) == set(before), 'Protected loose original path set changed'
assert {relative:stat_identity(file) for relative,file in after_files.items()} == metadata, 'Protected original file metadata changed'
assert all(sha(after_files[relative]) == before[relative]['sha256'] for relative in freshly_hashed), 'New loose record/report bytes changed'
assert all(sha(ROOT/relative) == digest for relative,digest in source_bindings.items()), 'Current production source/support bytes changed'
save(OUTPUT/'protected-files.json',before)
save(OUTPUT/'protected-verification.json',{'files':len(before),'completePathSetExact':True,'acceptedOriginalHashesRetained':True,'priorOriginalBytesRehashed':False,'priorProtectedIdentityLedger':{'path':str(PRIOR_PROTECTED),'sha256':PRIOR_LEDGER_SHA},'allFileStatIdentitiesUnchanged':True,'freshHashesBeforeAndAfter':freshly_hashed,'currentSourceAndSupportHashesExact':source_bindings,'sourcePackagingRepair':repair_binding})
for source,name in [(certificate,'completed-acceptance.json'),(supplement,'postclosure-readme-supplement.json'),
                    (Path(__file__).resolve(),'archive-owner.py'),(log_path,'utility.log'),
                    (work/'utility-terminal.json','utility-terminal.json'),
                    (closure_terminal,'docs-closure-terminal.json'),(Path(closure_actual['log']).resolve(),'docs-closure.log'),
                    (work/'protected-metadata-before.json','protected-metadata-before.json')]:
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
stats = {'gate':70,'records':records,'blobs':blobs,'databaseBytes':DATABASE.stat().st_size,
         'databaseSha256':migration['database']['sha256'],'databaseGrowthBytes':migration['databaseGrowthBytes'],
         'files':migration['files'],'artifactDirectories':paths,'originalLogicalBytes':migration['originalLogicalBytes'],
         'originalAllocatedBytes':migration['originalAllocatedBytes'],'grossAllocatedBytesReclaimed':migration['grossAllocatedBytesReclaimed'],
         'migrationEvidenceAllocatedBytes':evidence_bytes,'netAllocatedBytesReclaimedAfterEvidence':migration['grossAllocatedBytesReclaimed']-evidence_bytes,
         'cumulativeGrossAllocatedBytesReclaimed':cumulative,'protectedFilesExact':len(before),'priorOriginalBytesRehashed':False,'newLooseRecordsAndReportFreshlyHashed':len(freshly_hashed)}
stats_path.write_text(json.dumps(stats,indent=2)+'\n')
readme = ROOT/'benchmarks/storage/README.md'; history = ROOT/'benchmarks/storage/HISTORICAL_ARCHIVE.md'
guides_before = {str(path.relative_to(ROOT)):sha(path) for path in [readme,history]}
text = readme.read_text()
pattern = r'Completed historical/Gate55–69 migrations now store\n\*\*[^*]+\*\*\. Verified\nremoval reclaimed \*\*[^*]+\*\* before migration'
replacement = f'Completed historical/Gate55–70 migrations now store\n**{records:,} original records in {blobs:,} blobs, occupying {DATABASE.stat().st_size:,} bytes**. Verified\nremoval reclaimed **{cumulative:,} allocated bytes (about {cumulative/1048576:,.0f} MiB)** before migration'
text,count = re.subn(pattern,replacement,text); assert count == 1
old = '[Gate69](./closed-gate-69-migration/migration.json). Active gate evidence'
assert text.count(old) == 1
text = text.replace(old,'[Gate69](./closed-gate-69-migration/migration.json), and\n[Gate70](./closed-gate-70-migration/migration.json). Active gate evidence')
readme.write_text(text)
text = history.read_text(); assert '## Closed Gate70 artifacts' not in text
text += f'''\n\n## Closed Gate70 artifacts\n\nGate70's compiler, source, runtime and closure artifact directories are archived\nunder [the exact completed-gate plan](./closed-gate-70-migration/plan.json).\n[The executed migration](./closed-gate-70-migration/migration.json) preserves\n{migration['files']:,} original records, including complete String outputs,\nUTF-8 padding/source-order evidence, retained failed source packaging and its repair, and accepted actual command records. Independent decoding and scratch\nrestoration verified original paths, sizes and SHA256 before removal. All\n{len(before):,} protected top-level JSON/report paths and accepted byte identities remain unchanged.
Prior original hashes are retained from the accepted Gate69 ledger, with all file
stat identities unchanged before/after compaction; only the {len(new_loose_records)} new loose JSON
records and current report are freshly hashed.\nThe completed certificate and documentation supplement are retained with the\nmigration evidence. No tests or benchmark measurements repeated during compaction.\n\nOriginal artifacts occupied {migration['originalLogicalBytes']:,} logical bytes and\n{migration['originalAllocatedBytes']:,} allocated bytes. SQLite grew by\n{migration['databaseGrowthBytes']:,} bytes, reclaiming {migration['grossAllocatedBytesReclaimed']:,}\nallocated bytes gross and {stats['netAllocatedBytesReclaimedAfterEvidence']:,} bytes after\nmigration evidence, before guide growth. Restore only the four exact prefixes\nin the saved plan when historical consumers need original files.\n\nCurrent archive: **{records:,} records / {blobs:,} unique blobs / {DATABASE.stat().st_size:,} bytes**\n(about {DATABASE.stat().st_size/1048576:,.1f} MiB). Combined gross allocated savings total\n**{cumulative:,} bytes (about {cumulative/1048576:,.0f} MiB)** before evidence and\ndocumentation overhead. [The actual inventory](./closed-gate-70-migration/postmigration-stats.json)\nretains exact counts and sizes. Database SHA256 at this migration:\n`{migration['database']['sha256']}`. Linked raw results and top-level proofs remain loose.\n'''
history.write_text(text)
guide_record.write_text(json.dumps({'gate':70,'kind':'documentation-only-postcompaction-storage-guides',
    'utilityExitCode':code,'executedMigration':{'path':str(OUTPUT/'migration.json'),'sha256':sha(OUTPUT/'migration.json')},
    'files':[{'path':relative,'beforeSha256':digest,'afterSha256':sha(ROOT/relative)} for relative,digest in guides_before.items()],
    'testsRepeated':False,'measurementsRepeated':False},indent=2)+'\n')
assert evidence_bytes == sum(file.stat().st_blocks*512 for file in OUTPUT.iterdir() if file.is_file())
print(json.dumps(stats))
