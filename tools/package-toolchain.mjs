// Package already-built consumers; never start an implicit release build.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {execFileSync} from 'node:child_process';
const [compiler,server,target,output]=process.argv.slice(2);
if(!compiler||!server||!target||!output||process.argv.length!==6)throw Error('Usage: node tools/package-toolchain.mjs CLI LSP RUST_TARGET NEW_DIRECTORY');
if(!/^[a-z0-9_]+-[a-z0-9_-]+$/.test(target))throw Error('Use an explicit Rust target triple.');
const destination=path.resolve(output);fs.mkdirSync(destination);let complete=false;
try{
 fs.mkdirSync(path.join(destination,'bin'));
 const files={};
 for(const [name,input] of [['tsrust',compiler],['tsrust-lsp',server]]){
  const source=path.resolve(input);if(!fs.statSync(source).isFile())throw Error('Consumer must be a regular file.');
  const bytes=fs.readFileSync(source);files[name]={sha256:crypto.createHash('sha256').update(bytes).digest('hex'),bytes:bytes.length};fs.copyFileSync(source,path.join(destination,'bin',name));fs.chmodSync(path.join(destination,'bin',name),0o755);
 }
 const manifest={schemaVersion:1,target,rustToolchain:'1.96.1',sourceCommit:execFileSync('git',['rev-parse','HEAD'],{encoding:'utf8'}).trim(),sourceTreeDirty:execFileSync('git',['diff','--name-only'],{encoding:'utf8'}).trim().length>0,files};
 fs.writeFileSync(path.join(destination,'manifest.json'),JSON.stringify(manifest,null,2)+'\n');
 fs.writeFileSync(path.join(destination,'rust-toolchain.toml'),'[toolchain]\nchannel = "1.96.1"\nprofile = "minimal"\n');
 fs.writeFileSync(path.join(destination,'README.md'),'# Tsrust toolchain\n\nThis package contains the compiler and editor server for '+target+'. Verify each bin/ file against manifest.json before installation. Put both executables on PATH and configure the editor extension to use this tsrust-lsp. Rust 1.96.1 must already be installed to build native programs; checking and formatting do not need a Rust compiler. Native library sources are bundled in tsrust. Export a relocatable offline service with `tsrust emit main.tsr --package -o package`; migrations and application assets remain explicit deployment inputs.\n\nThis package is unsigned. Its hashes detect changes relative to the manifest, not an untrusted distributor. Build from the recorded source commit with the pinned Cargo.lock and toolchain when provenance matters. A dirty-source manifest is development evidence only, not a reproducible release identity.\n');
 for(const [name,record] of Object.entries(files)){const actual=crypto.createHash('sha256').update(fs.readFileSync(path.join(destination,'bin',name))).digest('hex');if(actual!==record.sha256)throw Error('Copied consumer digest mismatch');}
 complete=true;console.log(destination);
}finally{if(!complete)fs.rmSync(destination,{recursive:true,force:true});}
