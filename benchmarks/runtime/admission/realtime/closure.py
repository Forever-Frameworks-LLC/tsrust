"""Read prepared Cargo metadata/build JSON/lock; never invokes Cargo or fetches."""
import sys,json,pathlib,hashlib,tomllib,tarfile
metadata_path,artifacts_path,lock_path,output_path=sys.argv[1:]
metadata=json.loads(pathlib.Path(metadata_path).read_text());messages=[json.loads(line)for line in pathlib.Path(artifacts_path).read_text().splitlines()if line.strip()]
assert messages[-1]['reason']=='build-finished' and messages[-1]['success'] is True
nodes={n['id']:n for n in metadata['resolve']['nodes']};package_index={p['id']:p for p in metadata['packages']}
artifacts=[m for m in messages if m.get('reason')=='compiler-artifact'];active={a['package_id']for a in artifacts}
assert active<=set(nodes) and active<=set(package_index),'Unknown actual Cargo package'
assert sorted(next(a for a in artifacts if a.get('executable'))['features'])==['default','websocket','websocket-rustls'],'Direct root feature mismatch'
# Reuse HTTP SSE's actual-artifact proof: metadata optional/target paths are an
# upper bound. Do not fabricate runtime packages or features from those paths.
normal=set();pending=[metadata['resolve']['root']]
while pending:
 identity=pending.pop()
 if identity in normal or any('proc-macro'in t['kind']for t in package_index[identity]['targets']):continue
 normal.add(identity);pending += [d['pkg']for d in nodes[identity]['deps']if any(k['kind']is None for k in d['dep_kinds'])]
host=set();pending=[]
for artifact in artifacts:
 if 'proc-macro'in artifact['target']['kind']:pending.append(artifact['package_id'])
 pending += [d['pkg']for d in nodes[artifact['package_id']]['deps']if any(k['kind']=='build'for k in d['dep_kinds'])]
while pending:
 identity=pending.pop()
 if identity in host:continue
 host.add(identity);pending += [d['pkg']for d in nodes[identity]['deps']if any(k['kind']in(None,'build')for k in d['dep_kinds'])]
libraries=[a for a in artifacts if any(f.endswith('.rlib')for f in a['filenames'])]
runtime_artifacts=[a for a in libraries if a['package_id']in normal and str(a['profile']['opt_level'])=='3']
runtime={a['package_id']for a in runtime_artifacts};assert len(runtime)==len(runtime_artifacts),'Ambiguous optimized runtime artifact'
direct={d['pkg']for d in nodes[metadata['resolve']['root']]['deps']if any(k['kind']is None for k in d['dep_kinds'])}
assert direct<=runtime,'A direct runtime dependency lacks an optimized artifact'
for artifact in libraries:
 if artifact not in runtime_artifacts:assert artifact['package_id']in host,'Unoptimized library lacks a build/proc-macro path'
for artifact in artifacts:
 assert len(set(artifact['features']))==len(artifact['features'])
 assert set(artifact['features'])<=set(nodes[artifact['package_id']]['features']),'Compiled feature outside metadata upper bound'
runtime.add(metadata['resolve']['root'])
lock=tomllib.loads(pathlib.Path(lock_path).read_text());checksums={(p['name'],p['version']):p.get('checksum')for p in lock['package']};packages=[]
for package in metadata['packages']:
 if package['id']not in active:continue
 row={key:package.get(key)for key in['id','name','version','license','license_file','rust_version','source']};row['metadataFeatureUpperBound']=nodes[package['id']]['features'];selected=[a for a in artifacts if a['package_id']==package['id'] and (a in runtime_artifacts or a.get('executable'))];row['features']=selected[0]['features']if selected else sorted({feature for a in artifacts if a['package_id']==package['id']for feature in a['features']})
 if package.get('source','')and package['source'].startswith('registry+'):
  assert row['license']or row['license_file'];registry=pathlib.Path(package['manifest_path']).parents[1].name
  archive=pathlib.Path.home()/'.cargo/registry/cache'/registry/(package['name']+'-'+package['version']+'.crate');digest=hashlib.sha256(archive.read_bytes()).hexdigest();assert digest==checksums[(package['name'],package['version'])];row['archiveSha256']=digest;source_root=pathlib.Path(package['manifest_path']).parent;sidecar=source_root/'.cargo-checksum.json'
  if sidecar.exists():assert json.loads(sidecar.read_text())['package']==digest
  row['sourceFiles']={};expected_files=set()
  # Current Cargo caches may omit the checksum sidecar. Archive identity is still
  # pinned by Cargo.lock; compare every extracted byte and reject source extras.
  with tarfile.open(archive) as tar:
   for member in tar:
    if not member.isfile():continue
    relative=pathlib.PurePosixPath(member.name).relative_to(package['name']+'-'+package['version']);assert '..'not in relative.parts
    file=source_root/relative;data=tar.extractfile(member).read();assert file.read_bytes()==data,str(file);expected_files.add(str(relative));row['sourceFiles'][str(file)]=hashlib.sha256(data).hexdigest()
  actual_files={str(file.relative_to(source_root))for file in source_root.rglob('*')if file.is_file()}-{'.cargo-ok','.cargo-checksum.json'}
  assert actual_files==expected_files,'Unexpected or missing dependency source files: '+str(source_root)

 packages.append(row)
result={'kind':'prepared-real-active-closure','activePackageIds':sorted(active),'runtimePackageIds':sorted(runtime),'packages':packages,'lockSha256':hashlib.sha256(pathlib.Path(lock_path).read_bytes()).hexdigest(),'metadataSha256':hashlib.sha256(pathlib.Path(metadata_path).read_bytes()).hexdigest(),'artifactsSha256':hashlib.sha256(pathlib.Path(artifacts_path).read_bytes()).hexdigest()}
pathlib.Path(output_path).write_text(json.dumps(result,indent=2)+'\n')
