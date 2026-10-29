export const engineIDs = Object.freeze(['native','rust-hand','node-tsc','node-ts','tsx','ts-node-check','ts-node-transpile','bun','deno']);
export function measurementPlan(argv, manifest) {
 for (const arg of argv.filter(arg=>arg.startsWith('--'))) {
  if(arg!=='--verify'&&arg!=='--execution-only'&&arg!=='--allow-busy'&&!arg.startsWith('--engines=')&&!arg.startsWith('--only=')&&!arg.startsWith('--allocator-rlib=')) {
   throw new Error(`Unknown runtime option ${arg}; use --verify for correctness without timing`);
  }
 }
 const verify=argv.includes('--verify'), executionOnly=argv.includes('--execution-only');
 if(verify&&executionOnly) throw new Error('--verify and --execution-only cannot be combined');
 const selections=argv.filter(arg=>arg.startsWith('--engines='));
 if(selections.length>1) throw new Error('Specify --engines only once');
 // rust-hand (hand-written parity programs) runs only when selected explicitly.
 const requested=selections.length?selections[0].slice('--engines='.length).split(','):engineIDs.filter(id=>id!=='rust-hand');
 if(requested.some(id=>!engineIDs.includes(id))||new Set(requested).size!==requested.length) throw new Error('Expected unique known engine IDs in --engines');
 return {
  verify,
  engineIDs:engineIDs.filter(id=>requested.includes(id)),
  measurementScope:verify?'correctness-only':executionOnly?'execution-only':'runtime-startup-compilation',
  compilation:!verify&&!executionOnly,
  startupSamples:verify||executionOnly?0:manifest.startupSamples,
  processes:verify?0:manifest.processes,
  warmups:verify?0:manifest.warmups,
  samples:verify?0:manifest.samples,
  correctnessCases:iterations=>[[0,0],[1,7],[2,19],[17,3],[iterations,manifest.seed]]
 };
}
