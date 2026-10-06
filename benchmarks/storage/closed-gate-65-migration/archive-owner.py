# Source only until root authorizes after the actual completed certificate.
from pathlib import Path
import argparse, hashlib, json, shutil, sqlite3, subprocess, sys

ROOT = Path('/Users/seancotter/Desktop/dev/learning/tsrust')
HERE = Path(__file__).resolve().parent
OUTPUT = ROOT/'benchmarks/storage/closed-gate-65-migration'
DATABASE = ROOT/'benchmarks/records.sqlite'
parser = argparse.ArgumentParser()
for name in ['certificate', 'certificate-sha', 'readme-supplement', 'readme-supplement-sha']:
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
def snapshot():
    files = [ROOT/'BENCHMARKS.md']+[file for folder in [ROOT/'benchmarks/runtime/results', ROOT/'benchmarks/runtime/diagnostics'] for file in folder.glob('*.json')]
    return {str(file.relative_to(ROOT)):{'sha256':sha(file),'bytes':file.stat().st_size} for file in files}

certificate = Path(args.certificate).resolve(strict=True)
supplement = Path(args.readme_supplement).resolve(strict=True)
assert sha(certificate) == args.certificate_sha and sha(supplement) == args.readme_supplement_sha
closed, docs, plan = load(certificate), load(supplement), load(HERE/'plan.json')
assert closed['gate'] == 65 and closed['compilerAccepted'] and closed['runtimeAccepted']
assert closed['runtimeProof'] == plan['closedProof'] and docs['runtimeProof'] == plan['closedProof']
assert docs['gate'] == 65 and docs.get('measurementsRepeated') is False
runtime_path = Path(plan['closedProof']['path']).resolve(strict=True)
assert sha(runtime_path) == plan['closedProof']['sha256']
runtime = load(runtime_path)
assert runtime['scope']['allAfterOutputAndFinalDisposalsZero']
assert runtime['scope']['groupedReportMeasuredRows'] == 405
plan['completedAcceptance'] = {'path':str(certificate),'sha256':args.certificate_sha}
assert not OUTPUT.exists()
for relative in plan['paths']: assert (ROOT/'benchmarks'/relative).is_dir(), relative
actual_plan = HERE/'executed-plan.json'; save(actual_plan, plan)
before = snapshot(); save(HERE/'protected-before.json', before)
command = [sys.executable, str(ROOT/'benchmarks/storage/compact_closed_gate.py'), '--store', str(ROOT/'benchmarks/storage/store.py'), '--root', str(ROOT/'benchmarks'), '--database', str(DATABASE), '--plan', str(actual_plan), '--output', str(OUTPUT)]
subprocess.run(command, cwd=ROOT, check=True)
after = snapshot(); assert after == before, 'Protected original paths or bytes changed'
save(OUTPUT/'protected-files.json', before)
save(OUTPUT/'protected-verification.json', {'files':len(before),'completePathSetAndHashesExact':True})
shutil.copyfile(certificate, OUTPUT/'completed-acceptance.json')
shutil.copyfile(supplement, OUTPUT/'postclosure-readme-supplement.json')
shutil.copyfile(Path(__file__).resolve(), OUTPUT/'archive-owner.py')
migration = load(OUTPUT/'migration.json')
with sqlite3.connect(DATABASE.as_uri()+'?mode=ro', uri=True) as db:
    records = db.execute('SELECT count(*) FROM records').fetchone()[0]
    blobs = db.execute('SELECT count(*) FROM blobs').fetchone()[0]
stats_path = OUTPUT/'postmigration-stats.json'
stats_path.write_text('{}\n')
evidence_bytes = sum(file.stat().st_blocks*512 for file in OUTPUT.iterdir() if file.is_file())
stats = {'records':records,'blobs':blobs,'databaseBytes':DATABASE.stat().st_size,'databaseGrowthBytes':migration['databaseGrowthBytes'],'files':migration['files'],'originalLogicalBytes':migration['originalLogicalBytes'],'originalAllocatedBytes':migration['originalAllocatedBytes'],'grossAllocatedBytesReclaimed':migration['grossAllocatedBytesReclaimed'],'migrationEvidenceAllocatedBytes':evidence_bytes,'netAllocatedBytesReclaimedAfterEvidence':migration['grossAllocatedBytesReclaimed']-evidence_bytes,'protectedFilesExact':len(before)}
stats_path.write_text(json.dumps(stats,indent=2)+'\n')
assert evidence_bytes == sum(file.stat().st_blocks*512 for file in OUTPUT.iterdir() if file.is_file())
print(json.dumps(stats))
