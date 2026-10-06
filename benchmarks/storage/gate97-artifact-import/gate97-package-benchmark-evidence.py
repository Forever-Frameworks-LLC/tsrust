from pathlib import Path
import json,hashlib,shutil,datetime
root=Path('/Users/seancotter/Desktop/dev/learning/tsrust')
b=Path('/tmp/tsrust-gate97-nullable-callable-source-plan')
out=root/'benchmarks/runtime/diagnostics/gate97-benchmark-source-evidence'
assert not out.exists()
out.mkdir();(out/'objects').mkdir()
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def bound(p):return {'path':str(p),'sha256':sha(p),'bytes':p.stat().st_size}
selected=[]
for x in ['source-native-accepted.json','final-source-native-accepted.json','plan.json','acceptance-tools','counterpart-acceptance1','higher-order-counterpart-acceptance1','native-acceptance1','registration-proposal']:
 p=b/x;selected+=list(p.rglob('*')) if p.is_dir() else [p]
p=b/'parameter-repair'
for x in p.iterdir():
 if x.is_dir() and (x.name.startswith('counterpart-') or x.name.startswith('native-') and not x.name.endswith('-plan') or x.name in ['final13-retained-emit1','final14-all-emit1','final15-all-emit1']):
  selected+=list(x.rglob('*'))
 elif x.is_file():selected.append(x)
# Earlier source-only denominator/factory proposals are not executed evidence. Current and failed attempt snapshots are retained above.
records=[]
for p in sorted(set(selected)):
 if not p.is_file():continue
 assert not p.is_symlink()
 assert p.suffix not in ['.rlib','.rmeta','.o','.dylib']
 raw=p.read_bytes();digest=hashlib.sha256(raw).hexdigest();target=out/'objects'/digest
 if not target.exists():target.write_bytes(raw)
 assert sha(target)==digest
 records.append({'originalPath':str(p),'storedPath':str(target.relative_to(root)),'sha256':digest,'bytes':len(raw)})
receipt=root/'benchmarks/runtime/diagnostics/gate97-root-evidence/benchmark-registration.json'
link={'gate':97,'at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'scope':'Final15 source correspondence and guarded registration; no timing or full regression claim.',
 'epochOverlay':bound(b/'registration-proposal/final-epoch-overlay.json'),'registrationReceipt':bound(receipt),'sourceCandidate':bound(b/'registration-proposal/candidate-source.json'),
 'finalCorrespondence':bound(b/'parameter-repair/final15-all-emit1/correspondence.json'),'registeredSourceRows':691,'newRows':10,'sourceAccepted':True,'nativeSourceAccepted':True,'measured':False,'pendingMeasurementCheckpoint':101}
(out/'registration-receipt-linkage.json').write_text(json.dumps(link,indent=2)+'\n')
manifest={'gate':97,'scope':'Exact benchmark-owned counterpart/native acceptance and retained failed attempts, final compiler emission correspondence, and reviewed registration proposal. No timing or full regression claim. Root/compiler/editor bundles are separate; no compiler workspace, target, native executable or rlib copies.',
 'records':records,'files':len(records),'uniqueObjects':len(list((out/'objects').iterdir())),'logicalBytes':sum(r['bytes'] for r in records),'storedObjectBytes':sum(p.stat().st_size for p in (out/'objects').iterdir()),
 'currentAcceptance':bound(b/'final-source-native-accepted.json'),'finalCompilerCorrespondence':bound(b/'parameter-repair/final15-all-emit1/correspondence.json'),'registrationReceiptLinkage':'benchmarks/runtime/diagnostics/gate97-benchmark-source-evidence/registration-receipt-linkage.json',
 'excluded':'Unused source-only draft branches/earlier whole parent manifests, compiler workspaces/build caches/binaries. Actual compiler identities and binary hashes remain in recorded command bindings; owned compiler bytes are preserved by the compiler/root bundles.'}
(out/'restoration-map.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps({'path':str(out/'restoration-map.json'),'sha256':sha(out/'restoration-map.json'),'files':manifest['files'],'uniqueObjects':manifest['uniqueObjects'],'logicalBytes':manifest['logicalBytes'],'storedObjectBytes':manifest['storedObjectBytes'],'linkage':bound(out/'registration-receipt-linkage.json')},indent=2))
