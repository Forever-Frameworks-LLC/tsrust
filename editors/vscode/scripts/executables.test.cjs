const test=require('node:test'),assert=require('node:assert/strict'),fs=require('node:fs'),os=require('node:os'),path=require('node:path');
const temp=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-discovery-'));
test.after(()=>fs.rmSync(temp,{recursive:true,force:true}));
require('esbuild').buildSync({entryPoints:['src/executables.ts'],bundle:true,platform:'node',format:'cjs',outfile:path.join(temp,'resolver.cjs')});
const {resolveExecutable}=require(path.join(temp,'resolver.cjs'));
function executable(file){fs.mkdirSync(path.dirname(file),{recursive:true});fs.writeFileSync(file,'#!/bin/sh\nexit 0\n',{mode:0o755});return file;}
test('new projects discover Cargo executables without workspace settings',async()=>{
 const home=path.join(temp,'home'),file=executable(path.join(home,'.cargo/bin/tsrust-lsp'));
 assert.equal(await resolveExecutable('tsrust-lsp',{home,envPath:''}),file);
});
test('explicit invalid paths fail; sibling override precedes global discovery',async()=>{
 const home=path.join(temp,'home'),file=executable(path.join(temp,'custom/tsrust-lsp'));
 assert.equal(await resolveExecutable('tsrust-lsp',{home,envPath:'',companion:path.join(temp,'custom/tsrust')}),file);
 await assert.rejects(resolveExecutable('tsrust-lsp',{home,configured:path.join(temp,'missing'),envPath:''}));
 await assert.rejects(resolveExecutable('tsrust-lsp',{home,configured:'relative',envPath:''}));
});
test('absolute PATH entries work; relative entries and directories are excluded',async()=>{
 const home=path.join(temp,'empty'),file=executable(path.join(temp,'path/tsrust'));
 assert.equal(await resolveExecutable('tsrust',{home,envPath:path.dirname(file)}),file);
 await assert.rejects(resolveExecutable('tsrust',{home,envPath:'.'}));
 fs.mkdirSync(path.join(temp,'directory/tsrust'),{recursive:true});
 await assert.rejects(resolveExecutable('tsrust',{home,envPath:path.join(temp,'directory')}));
});
