import test from 'node:test';
import assert from 'node:assert/strict';
import path from 'node:path';
import {readFileSync, writeFileSync} from 'node:fs';
import {root, compiler, run, temporary, formattedWitnesses, execute, reducedIdentityLimit, tokens} from '../harness.mjs';

function source(dir, name, text) {
  const file=path.join(dir,name); writeFileSync(file,text);
  const formatted=run(compiler,['format',file]).stdout;
  assert.deepEqual(tokens(formatted),tokens(text));writeFileSync(file,formatted);
  assert.equal(run(compiler,['format',file]).stdout,formatted);return file;
}

test('I03-I09,I11-I12: owned allocator, all foreign operations and recreation execute',()=>temporary(dir=>{
  formattedWitnesses(dir,assert);
  const file=path.join(dir,'identity.tsr');
  assert.equal(execute(file,dir),'PASS identity: independent runtimes, all foreign paths, copied handles, FIFO, retirement and recreation\n');
  const checked=run(compiler,['check',file]).stdout;
  assert.match(checked,/runtime[^\n]*: mutate, owned escape\n\s+domain: storage consume, access mutate/);
  const emitted=readFileSync(path.join(dir,'witness.rs'),'utf8');
  // The std:testing resource and Result types precede Domain and Handle in this fixed witness module.
  const fields=emitted.match(/struct Record3 \{([^}]*)\}/)?.[1].trim();
  assert.equal(fields,'field0: i64, field1: i64, field2: i64');
}));

test('I10: same allocation algorithm at reduced ceiling rejects without wrap or state mutation',()=>temporary(dir=>{
  formattedWitnesses(dir,assert); reducedIdentityLimit(dir);
  assert.equal(execute(path.join(dir,'identity-exhaustion.tsr'),dir),'PASS identity exhaustion: last runtime/owner/instance; repeated rejection preserves state\n');
  const owner=readFileSync(path.join(dir,'owner.tsr'),'utf8');
  for (const counter of ['nextRuntime','domain.next','owner.next']) {
    const guard=owner.indexOf(`if (${counter} >= 3)`);
    const update=owner.indexOf(`${counter} = ${counter} + 1`);
    assert.ok(guard>=0 && update>guard, counter);
    assert.match(owner.slice(guard,update),/throw '(runtime|owner|instance) identity exhausted'/);
  }
}));

test('I12,I15: public construction cannot clone allocators/owners or import private constructors',()=>temporary(dir=>{
  formattedWitnesses(dir,assert);
  const cases=[
    ["import {runtime} from './owner.tsr';function main(){const allocate=runtime();const copy=clone(allocate);copy();}",'shared aliases are unsupported'],
    ["import {runtime} from './owner.tsr';function main(){const allocate=runtime();const moved=allocate;allocate();moved();}",'transferred'],
    ["import {runtime} from './owner.tsr';function main(){const allocate=runtime();const owner=allocate();const copy=clone(owner);}",'clone'],
    ["import {create} from './owner.tsr';function main(){}",'export'],
    ["import {Domain} from './owner.tsr';function main(){}",'export'],
  ];
  for(const [text,diagnostic] of cases) {
    const file=source(dir,'negative.tsr',text);
    for(const command of ['check','emit']) {
      const result=run(compiler,[command,file],1);
      assert.match(result.stderr,new RegExp(diagnostic,'i'));
      assert.doesNotMatch(result.stderr,/panicked|stack overflow/);
    }
  }
  const control=source(dir,'control.tsr',"import {strictEqual} from 'std:testing';import {runtime,root,valid} from './owner.tsr';function main(){const allocate=runtime();const moved=allocate;const owner=moved();const h=root(owner);const copy=clone(h);strictEqual(valid(owner,copy),true);const other=moved();strictEqual(valid(other,copy),false);}");
  assert.equal(execute(control,dir),'');
}));

test('I12: diamond imports share runtime allocation; returned allocator can move across helpers',()=>temporary(dir=>{
  formattedWitnesses(dir,assert);
  source(dir,'left.tsr',"import {runtime,Owner} from './owner.tsr';export function left():()=>Owner{return runtime();}");
  source(dir,'right.tsr',"import {runtime,Owner} from '././owner.tsr';export function right():()=>Owner{return runtime();}");
  const file=source(dir,'imports.tsr',"import {strictEqual} from 'std:testing';import {left} from './left.tsr';import {right} from './right.tsr';import {Owner,root,valid} from './owner.tsr';function move(allocator:()=>Owner):()=>Owner{return allocator;}function main(){const a=left();const b=right();const moved=move(a);const first=moved();const second=b();strictEqual(first.id,second.id);strictEqual(first.runtime!==second.runtime,true);const h=root(first);strictEqual(valid(second,h),false);}");
  assert.equal(execute(file,dir),'');
}));

test('I18-I20: complete identity matrix and reconciled historical statuses',()=>{
  const matrix=JSON.parse(readFileSync(path.join(root,'identity-matrix.json'),'utf8'));
  assert.deepEqual(matrix.rows.map(row=>row.id),Array.from({length:20},(_,i)=>`I${String(i+1).padStart(2,'0')}`));
  assert.equal(matrix.investigation,'complete');assert.equal(matrix.identity,'enforced within documented source API');
  for(const row of matrix.rows) {
    assert.ok(['executed','analysis/emission only','expected rejection','blocked','not applicable with reason'].includes(row.status));
    assert.ok(row.evidence && row.witness && row.commands.length);
    assert.notEqual(row.status,'blocked',row.id);
  }
  const h=JSON.parse(readFileSync(path.join(root,'matrix.json'),'utf8'));
  const r=JSON.parse(readFileSync(path.join(root,'resource-matrix.json'),'utf8'));
  assert.equal(h.blockers.B1.status,'resolved by enforced source API');
  assert.equal(h.rows.find(row=>row.id==='H02').status,'executed');
  assert.equal(h.rows.find(row=>row.id==='H10').status,'blocked');
  for(const row of h.rows) assert.deepEqual(r.reconciliation[row.id],{status:row.status,evidence:row.evidence});
  assert.deepEqual(matrix.reconciliation.H,h.rows.map(row=>row.id));
  assert.deepEqual(matrix.reconciliation.R,r.rows.map(row=>row.id));
});

test('I19: demo usage and unexpected tool errors exit1 rather than the contract-blocked code',()=>{
  const cli=path.join(root,'cli.mjs');
  assert.match(run(process.execPath,[cli],1).stderr,/Usage:/);
  const program=`process.env.TSRUST='/nonexistent/ui-identity-compiler';process.argv=['node','cli','demo'];await import(${JSON.stringify(cli)});`;
  assert.match(run(process.execPath,['--input-type=module','-e',program],1).stderr,/ENOENT/);
});
