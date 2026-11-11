"""Prove installed ws files against a supplied official npm tarball/lock; no fetch/extraction."""
import sys,pathlib,tarfile,json,hashlib,base64
archive,lock,installed,output=map(pathlib.Path,sys.argv[1:]);raw=archive.read_bytes();data=json.loads(lock.read_text());entry=data['packages']['node_modules/ws'];assert entry['version']=='8.18.3';integrity='sha512-'+base64.b64encode(hashlib.sha512(raw).digest()).decode();assert entry['integrity']==integrity
files={}
with tarfile.open(archive,mode='r:gz')as package:
 for member in package:
  if not member.isfile():continue
  name=pathlib.PurePosixPath(member.name);assert not name.is_absolute()and'..'not in name.parts and name.parts[0]=='package';relative=pathlib.Path(*name.parts[1:]);body=package.extractfile(member).read();actual=installed/relative;assert actual.read_bytes()==body,str(relative);files[str(actual.resolve())]=hashlib.sha256(body).hexdigest()
actual={str(p.resolve())for p in installed.rglob('*')if p.is_file()};assert actual==set(files);metadata=json.loads((installed/'package.json').read_text());assert metadata['name']=='ws'and metadata['version']=='8.18.3'and metadata['license']=='MIT'
output.write_text(json.dumps({'kind':'verified-node-package','name':'ws','version':'8.18.3','license':'MIT','lockSha256':hashlib.sha256(lock.read_bytes()).hexdigest(),'archiveSha256':hashlib.sha256(raw).hexdigest(),'integrity':integrity,'files':files},indent=2)+'\n')
