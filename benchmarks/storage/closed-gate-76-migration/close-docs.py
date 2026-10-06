"""Root executes only after accepted compiler, source, runtime and profile proofs."""
from pathlib import Path
import argparse,hashlib,json,math,subprocess
root=Path('/Users/seancotter/Desktop/dev/learning/tsrust');here=Path(__file__).resolve().parent
sha=lambda path:hashlib.sha256(Path(path).read_bytes()).hexdigest()
load=lambda path:json.loads(Path(path).read_text())
p=argparse.ArgumentParser();p.add_argument('--runtime-proof',required=True);p.add_argument('--runtime-sha256',required=True);p.add_argument('--certificate',default=str(here/'completed-acceptance.json'));a=p.parse_args()
runtime_path=Path(a.runtime_proof).resolve(strict=True);certificate_path=Path(a.certificate).resolve();assert not certificate_path.exists()
assert sha(runtime_path)==a.runtime_sha256
binding={'path':str(runtime_path),'sha256':a.runtime_sha256};r=load(runtime_path)
def accepted(item):
 path=Path(item['path']);path=path if path.is_absolute() else root/path
 assert sha(path)==item['sha256'];return load(path)
c=accepted(r['compilerProof']);s=accepted(r['sourceProof'])
assert r['gate']==c['gate']==s['gate']==76 and r['runtimeAccepted'] and c['compilerAccepted'] and s['sourceAccepted']
counts=r['scope'];assert (counts['newWorkloads'],counts['engines'],counts['processCells'],counts['timedBatches'],counts['publishedRateCells'],counts['nativeProfileSamples'],counts['groupedReportMeasuredRows'])==(4,8,96,2976,32,12,488)
assert counts['allDisposalsZero'] and counts['old484SectionsByteExact'] and counts['old9MemoryTablesByteExact'] and counts['auxiliaryTablesByteExact'] and not counts['oldMeasurementsRepeated']
regression=c['regression'];assert regression['accepted'] and regression['fullAttempts']==1 and regression['fullRestarts']==0
compiler_groups,workspace_groups=regression['uniqueGroups'],regression['totalWorkspaceGroups'];assert (compiler_groups,workspace_groups)==(661,680)
assert c['focusedAcceptance']['focusedAccepted'] and c['focusedAcceptance']['groups']==4 and all(x['exitCode']==0 for x in c['focusedAcceptance']['attempts'])
assert all(x['exitCode']==0 for x in c['finalChecks']) and c['freshCLI']['freshDebugReleaseByteExact']
assert c['freshCLI']['releaseSha256']==r['acceptedReleaseBinarySha256']==sha(root/'target/release/tsrust')
assert c['nativePoolAndOrder']['nativeBuilds']==4 and c['nativePoolAndOrder']['completeOutputLines']==96
assert c['unicodeBoundary']['accepted'] and c['unicodeBoundary']['invalidByteSourcesRejectedBeforeParsing']==4
assert s['productionIntegrated'] and s['sourceEntries']==488 and s['priorRowsPreserved']==484 and s['priorManifestObjectsExact']
assert (s['scope']['rows'],s['scope']['cases'],s['scope']['handCells'],s['scope']['negative'],s['scope']['actualSemanticMutantsRejected'])==(4,548,48,89,5)
assert sha(root/'BENCHMARKS.md')==r['reportSha256']
manifest_path=root/'benchmarks/runtime/manifest.json';manifest=load(manifest_path);assert len(manifest['workloads'])==488
ids={'string_is_well_formed_ascii','string_is_well_formed_unicode','string_to_well_formed_ascii','string_to_well_formed_unicode'}
assert len(r['rates'])==4 and {x['id'] for x in r['rates']}==ids
assert all(math.isfinite(x[k]) and x[k]>0 for x in r['rates'] for k in ['nativeNode','nativeFastest'])
assert len(r['normalizedNativeResources'])==4
assert all(x['samples']==3 and x['operationsPerSample']==2048 and x['liveBytesAfterOutputDrop']==x['liveBytesAfterDrop']==0 for x in r['normalizedNativeResources'])
node=sum(x['nativeNode']>1 for x in r['rates']);fast=sum(x['nativeFastest']>1 for x in r['rates'])
ratio=f'Native leads compiled Node on {node}/4 rows and the fastest scripting runtime on {fast}/4.'
resource='Twelve profiles retain raw counters and normalized min/average/max allocations, reallocations and requested/retained bytes across three 2,048-operation samples per row. Input adaptation is separate. Peak is retained-batch peak divided by operations, not an isolated-call peak; all output and final disposal counters return to zero. Predicate allocation witnesses observe zero method allocations; owned conversion copies naturally, while JavaScript may share string storage. No allocation parity is claimed.'
paths=['docs/milestones/76-native-string-well-formedness.md','docs/ROADMAP.md','docs/README.md','crates/tsrust-compiler/README.md','benchmarks/runtime/README.md','benchmarks/coverage/status.json','benchmarks/coverage/inventory.json','docs/feature-coverage.md']
before={name:sha(root/name) for name in paths};texts={name:(root/name).read_text() for name in paths[:5]}
old='Status: implementation in progress. Gate 75 remains the latest completed gate.';assert old in texts[paths[0]]
assert texts[paths[1]].splitlines()[0].startswith('**Current native String gate:**')
index_old='[Milestone 76: native String well-formedness](./milestones/76-native-string-well-formedness.md) is in progress: once-only receivers, valid UTF-8 guarantees and standard Cow ownership.';assert index_old in texts[paths[2]]
anchor='- Primitive zero-argument';assert texts[paths[3]].count(anchor)==1
assert texts[paths[4]].count('The current 484 workload pairs')==1
status=load(root/paths[5]);prior_status=json.loads(json.dumps(status));inventory=load(root/paths[6])['entries'];assert len(inventory)==1926 and len({x['category'] for x in inventory})==28
assert {'String.isWellFormed','String.toWellFormed'}<={x['id'] for x in inventory}
texts[paths[0]]=texts[paths[0]].replace(old,f'Status: complete. {compiler_groups} compiler groups, {workspace_groups} workspace groups, four matched comparisons and twelve fully disposed profiles pass.')
texts[paths[0]]+='\n## Executed acceptance\n\nFour focused compiler groups pass first attempt at O0/O3, including used and unused receiver effects, trapping receivers, incoming loans, captures, selected temporary consumers and fixed storage. Unicode witnesses cover all scalar widths, U+10FFFF, NUL, BOM, replacement characters, combining sequences, noncharacters and astral charAt/split outputs. Observed, black-boxed resource witnesses verify predicate zero allocations, static borrowing, independent owned Cow copies and normal/unwind disposal. The existing safe String Cow helper is reused unchanged. Strict arity/type and ordinary-stack depth diagnostics pass. Editor completion, zero-parameter signatures and Boolean/String result chains pass.\n\nOne full workspace regression accepts '+str(compiler_groups)+' compiler groups and '+str(workspace_groups)+' total workspace groups, without repairs or a restart. Doctests, formatting, Clippy, debug/release builds and fresh byte-exact emission pass. Source validation retains two Node groups, 548 complete cases, 48 literal hands, 89 negatives and five executed semantic mutants. Four native O0/O3 builds reproduce 96 complete pool/order lines. Four malformed UTF-8 CLI inputs are rejected at the existing source boundary; no decoder or surrogate policy changes.\n\nThe execution-only run retains 96 process cells, 2,976 timed batches and 32 rate cells across eight engines. Pool construction, dynamic receiver selection, the actual methods and natural Boolean[]/String[] retention are timed; complete observation is outside timing. The native predicate exploits the valid UTF-8 invariant. Conversion copies owned native strings, so these whole-workload rates do not describe JavaScript surrogate repair or isolated method latency. '+ratio+' '+resource+' Earlier 484 runtime sections, nine memory tables and auxiliary tables remain exact and are not retimed. The report contains 488 separately dated rows.\n\n'
for label,item in [('compiler proof',r['compilerProof']),('source proof',r['sourceProof']),('runtime and resource proof',binding)]:
 path=Path(item['path']);path=path if path.is_absolute() else root/path
 texts[paths[0]]+=f'The [{label}](../../{path.relative_to(root)}) retains original inputs, commands and outputs.\n\n'
line=texts[paths[1]].splitlines()[0];texts[paths[1]]=texts[paths[1]].replace(line,'**Completed native String gate:** [Milestone 76](./milestones/76-native-string-well-formedness.md) accepts strict zero-argument well-formedness methods through valid UTF-8 and standard Cow ownership. '+str(compiler_groups)+' compiler groups and '+str(workspace_groups)+' workspace groups pass in one full regression; four comparisons and twelve disposed profiles pass. '+ratio,1).replace('through gate 75','through gate 76')
texts[paths[2]]=texts[paths[2]].replace(index_old,'[Milestone 76: native String well-formedness](./milestones/76-native-string-well-formedness.md) is complete: once-only borrowed receivers, valid UTF-8 guarantees, standard Cow ownership, '+str(compiler_groups)+' compiler groups, '+str(workspace_groups)+' workspace groups, four comparisons and twelve disposed profiles.')
texts[paths[3]]=texts[paths[3]].replace(anchor,'- Zero-argument String `isWellFormed()` evaluates and borrows its receiver once, then returns true because every native string is valid UTF-8. Receiver effects, traps and temporary disposal remain observable even when the result is unused. `toWellFormed()` reuses standard Cow::clone: static text remains borrowed, nonempty owned text copies independently, and owned empty text needs no allocation. Lone surrogates and JavaScript coercion are outside the native representation. [Milestone 76](../../docs/milestones/76-native-string-well-formedness.md) records four focused groups, editor support, one full regression, four whole-workload comparisons and disposal.\n'+anchor,1)
texts[paths[4]]=texts[paths[4]].replace('The current 484 workload pairs','The current 488 workload pairs')
texts[paths[4]]+='\nGate76 adds four ASCII/Unicode String.isWellFormed/toWellFormed comparisons with complete Boolean[] or String[] results. Dynamic pool setup, receiver selection, method execution and retention are timed; observation is outside timing. Native valid UTF-8 makes the predicate true, while standard Cow cloning preserves static borrowing and independently copies owned text. JavaScript lone-surrogate repair is outside this common input domain. '+ratio+' '+resource+' Prior 484 sections, nine memory tables and auxiliary baselines remain exact. See [Milestone 76](../../docs/milestones/76-native-string-well-formedness.md).\n'
for key,method in [('String.isWellFormed','is_well_formed'),('String.toWellFormed','to_well_formed')]:
 status[key]={'implementation':'native variant','comparison':'paired group','workloads':sorted(x for x in ids if x.startswith('string_'+method+'_')),'evidence':['./milestones/76-native-string-well-formedness.md'],'note':('Strict zero-argument borrowed String receiver evaluated once, with effects, traps, loans and temporary disposal preserved. Safe native UTF-8 guarantees a true predicate without a validation scan. No representable lone surrogates or JavaScript coercion. Four focused groups, editor/source/native/runtime and disposal acceptance complete.' if method=='is_well_formed' else 'Strict zero-argument borrowed String receiver evaluated once; content remains exact through standard Cow clone. Static storage stays borrowed, nonempty owned storage copies independently, and owned empty cloning allocates nothing. Valid native UTF-8 has no representable lone surrogates; no JavaScript coercion or allocation parity claim. Complete-result source/native/runtime and disposal acceptance complete.')}
assert all(status[k]==value for k,value in prior_status.items() if k not in ['String.isWellFormed','String.toWellFormed'])
for name,text in texts.items():(root/name).write_text(text)
(root/paths[5]).write_text(json.dumps(status,indent=2)+'\n')
subprocess.run(['/Users/seancotter/.nvm/versions/node/v24.11.1/bin/node','benchmarks/coverage/update.mjs'],cwd=root,check=True)
inventory=load(root/paths[6])['entries'];assert len(inventory)==1926 and len({x['category'] for x in inventory})==28
assert sha(root/'BENCHMARKS.md')==r['reportSha256']
certificate={'gate':76,'compilerAccepted':True,'runtimeAccepted':True,'namedGroups':compiler_groups,'workspaceNamedGroups':workspace_groups,'measuredRows':488,'releasePath':str(root/'target/release/tsrust'),'releaseSha256':r['acceptedReleaseBinarySha256'],'compilerProof':r['compilerProof'],'sourceProof':r['sourceProof'],'runtimeProof':binding,'manifest':{'path':str(manifest_path),'sha256':sha(manifest_path)},'report':{'path':str(root/'BENCHMARKS.md'),'sha256':r['reportSha256']},'inventoryOperations':1926,'inventoryFamilies':28,'rateLeads':{'nativeNode':node,'nativeFastest':fast,'rows':4},'docsChanges':[{'path':name,'beforeSha256':before[name],'afterSha256':sha(root/name)} for name in paths],'fullRestarts':regression['fullRestarts']}
with certificate_path.open('x') as out:json.dump(certificate,out,indent=2);out.write('\n')
print(json.dumps({'certificate':str(certificate_path),'rateLeads':certificate['rateLeads'],'compilerGroups':compiler_groups,'workspaceGroups':workspace_groups}))
