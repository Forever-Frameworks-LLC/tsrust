// Prepare compiler-owned dependencies outside timed rustc compilation. Cargo's
// artifact messages identify exact libraries; callers never guess cache hashes.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
const names=['serde','serde_json'];
const hash=bytes=>crypto.createHash('sha256').update(bytes).digest('hex');
export function dependencyArtifacts(stdout,target,requiredNames=names) {
 const result={};
 for(const line of stdout.split('\n').filter(Boolean)) {
  const row=JSON.parse(line),name=row.target?.name;
  if(row.reason!=='compiler-artifact'||!requiredNames.includes(name)||!row.target.kind.includes('lib'))continue;
  for(const file of row.filenames.filter(file=>file.endsWith('.rlib'))) {
   const relative=path.relative(target,file);
   if(relative==='..'||relative.startsWith('..'+path.sep)||path.isAbsolute(relative))throw new Error(`Dependency artifact outside target: ${file}`);
   if(result[name]&&result[name]!==file)throw new Error(`Ambiguous dependency artifact: ${name}`);
   result[name]=file;
  }
 }
 for(const name of requiredNames)if(!result[name])throw new Error(`Missing dependency artifact: ${name}`);
 return result;
}
export function needsJsonDependencies(emitted) { return /(?:^|[;}]\s*)mod\s+tsrust_native_json\s*\{/m.test(emitted); }
export function prepareNativeDependencies({emitted,compiler,source,build,rustVersion,targetDirectory,env=process.env}) {
 if(!needsJsonDependencies(emitted))return {args:[],evidence:null};
 // Stable source paths keep rebuilt dependency artifacts identical across A/B runs.
 const scratch=path.join(build,'package-native-dependencies');
 fs.mkdirSync(scratch); // Fail closed if another preparation owns this directory.
 const packageRoot=path.join(scratch,'source');
 const target=targetDirectory??(env.CARGO_TARGET_DIR?path.resolve(env.CARGO_TARGET_DIR):path.join(build,'native-dependencies'));
 // Export reads prepared portable sources from the caller's Cargo home; the
 // isolated build home below must not hide them.
 const callerHome=env.CARGO_HOME?path.resolve(env.CARGO_HOME):path.join(process.env.HOME??'','.cargo');
 const sourcePacks=env.TSRUST_SOURCE_PACK_DIR??path.join(callerHome,'tsrust','source-packs');
 const childEnv={...env,CARGO_HOME:path.join(scratch,'cargo-home'),CARGO_TARGET_DIR:target,CARGO_INCREMENTAL:'0',TSRUST_SOURCE_PACK_DIR:sourcePacks};
 // Do not inherit profile overrides, wrappers or flags that change measured code.
 for(const key of Object.keys(childEnv))if(key==='RUSTFLAGS'||key==='CARGO_ENCODED_RUSTFLAGS'||key==='RUSTC'||key.startsWith('RUSTC_')||key.startsWith('CARGO_PROFILE_')||key.startsWith('CARGO_BUILD_')||key.startsWith('CARGO_TARGET_')&&key!=='CARGO_TARGET_DIR')delete childEnv[key];
 childEnv.CARGO_BUILD_JOBS='1';
 const run=(command,args,cwd)=>{
  const child=spawnSync(command,args,{cwd,env:childEnv,encoding:'utf8',timeout:120000,maxBuffer:16*1024*1024});
  if(child.error||child.status!==0)throw new Error(`Native dependency preparation failed: ${child.error??child.stderr+child.stdout}`);
  return child.stdout;
 };
 try {
  run(compiler,['emit',source,'--package','-o',packageRoot],build);
  const packaged=fs.readFileSync(path.join(packageRoot,'src/main.rs'),'utf8');
  if(packaged.trimEnd()!==emitted.trimEnd())throw new Error('Exported source differs from benchmark emission');
  const command=['rustup','run',rustVersion,'cargo','build','--offline','--locked','--release','--message-format=json'];
  const artifacts=dependencyArtifacts(run(command[0],command.slice(1),packageRoot),target);
  const args=[],directories=new Set(),libraries=[];
  for(const name of names){const file=artifacts[name];args.push('--extern',`${name}=${file}`);directories.add(path.dirname(file));libraries.push({name,path:file,sha256:hash(fs.readFileSync(file))});}
  for(const directory of directories)args.push('-L',`dependency=${directory}`);
  return {args,evidence:{preparation:'untimed-offline-cargo',command,rustVersion,manifestSha256:hash(fs.readFileSync(path.join(packageRoot,'Cargo.toml'))),lockSha256:hash(fs.readFileSync(path.join(packageRoot,'Cargo.lock'))),configSha256:hash(fs.readFileSync(path.join(packageRoot,'.cargo/config.toml'))),libraries}};
 } finally {fs.rmSync(scratch,{recursive:true,force:true});}
}

// Reference-only crates never change the emitted compiler or Node dependency path.
export function prepareReferenceDependencies({workload,rustVersion,targetDirectory,env=process.env}) {
 const selected={math_random_range:'rand',string_normalize_forms:'unicode_normalization'}[workload?.id];
 if(!selected)return {args:[],evidence:null};
 const root=fileURLToPath(new URL('./parity-dependencies/',import.meta.url));
 const target=path.resolve(targetDirectory),childEnv={...env,CARGO_TARGET_DIR:target,CARGO_INCREMENTAL:'0',CARGO_BUILD_JOBS:'1'};
 for(const key of Object.keys(childEnv))if(key==='RUSTFLAGS'||key==='CARGO_ENCODED_RUSTFLAGS'||key==='RUSTC'||key.startsWith('RUSTC_')||key.startsWith('CARGO_PROFILE_')||key.startsWith('CARGO_BUILD_')&&key!=='CARGO_BUILD_JOBS'||key.startsWith('CARGO_TARGET_')&&key!=='CARGO_TARGET_DIR')delete childEnv[key];
 const command=['rustup','run',rustVersion,'cargo','build','--manifest-path',path.join(root,'Cargo.toml'),'--offline','--locked','--release','--message-format=json'];
 const child=spawnSync(command[0],command.slice(1),{cwd:root,env:childEnv,encoding:'utf8',timeout:120000,maxBuffer:16*1024*1024});
 if(child.error||child.status!==0)throw Error(`Reference dependency preparation failed: ${child.error??child.stderr+child.stdout}`);
 const file=dependencyArtifacts(child.stdout,target,[selected])[selected];
 return {args:['--extern',`${selected}=${file}`,'-L',`dependency=${path.dirname(file)}`],evidence:{preparation:'untimed-offline-reference-cargo',rustVersion,manifestSha256:hash(fs.readFileSync(path.join(root,'Cargo.toml'))),lockSha256:hash(fs.readFileSync(path.join(root,'Cargo.lock'))),libraries:[{name:selected,sha256:hash(fs.readFileSync(file))}]}};
}
