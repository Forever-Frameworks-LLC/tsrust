# Once callback environment admission

The owner chose to try existing Once syntax before adding a fourth wrapper.
Invocation access and environment lifetime are separate requirements. A callback
can consume its owned record while borrowing a local array. Immediate invocation
must accept it; retaining that environment requires ownership of its captures.
ReadOnly, Mutable and Once remain the public invocation vocabulary.

## Evidence and rejected experiment

The scoped parameter witness fails at 3a7eaf1 because public Once resolves to
OwnedOnce. Changing all Once parameters to internal Once permits direct closures,
conditional borrowed families and joined callback failures. It also breaks two
existing callback_moves tests: returning a moved callback and returning a closure
that retains the callback. The experiment is reverted. Those positive tests must
remain accepted; rejecting all parameter escapes would change existing behavior
and is not an acceptable solution. The new owning-retention witness verifies direct
Once return and rejects a borrowed conditional family passed to that retainer.

## Required implementation

Add an isolated environment-demand analysis using source binding identities and
checked callable origin facts. Keep the existing distinction between scoped Once
and OwnedOnce internally. Do not change public annotation parsing globally.

For each concrete function parameter, derive whether the environment is invoked
or forwarded within scope, or retained in a returned callback, returned aggregate,
owned field, registry, thrown payload or other escaping destination. Declaration
moves and assignments carry the original parameter identity. Lexical shadowing
must not confuse separate bindings. Conditional origins and branch joins retain
all possible demands. Local capture is not automatically escape: trace the
capturing environment's actual lifetime and disposition.

Propagate demands through calls and callback parameter forwarding with a monotone
worklist. Recursive groups converge; missing or opaque destinations cannot certify
non-escape. Preserve default/rest argument mapping, optional calls, failure exits
and finally completion. Existing source/native capture facts remain authoritative;
do not infer environment ownership solely from consuming invocation access.

Resolve a concrete function's Once parameter to the scoped internal contract only
when its body and forwarded destinations prove no retention. Retainers continue
to use OwnedOnce and the native static environment bound. Contract identities and
native generic adapters must preserve this environment demand independently of
public type spelling. Anonymous higher-order signatures without a verified body
must retain an owning requirement until origin-specific non-escape evidence exists.
Do not silently interchange their shapes merely because both render as Once.

## Acceptance gates

1. Establish failing borrowed direct/conditional and forwarding witnesses alongside
   positive existing owned retention and caller-side borrowed escape rejections.
2. Implement the isolated demand kernel with binding identity, alias, shadowing,
   branch and recursive forwarding tests, including conservative unknown controls.
3. Integrate concrete signature resolution and native lifetime bounds. Verify direct
   invocation, defaults/rest, callback failures, forwarding, owned factory returns,
   record storage and abandoned/invoked disposal at optimization levels 0 and 3.
4. Export inferred behavior to editor hover/help without adding syntax. Preserve
   result-chain help and diagnose unsafe retention at the argument or escape site.
5. Extend the matched workload with a scoped Once parameter call and an owned
   retaining factory. Keep independent oracle observations, source guards, mutation
   rejection, registrations and source fingerprints ready. Do not claim timing.
6. Run the affected compiler suites, eight depth canaries and whole editor crate.
   Format the entire project, store exact evidence in SQLite, update current gate
   records and commit accepted work with explicit paths. Refresh consumers from
   the accepted checkpoint atomically.

Gate173 and the broader callable/environment plan remain open. This plan supplies
the next implementation boundary; it does not replace scoped aggregate/builtin
transport, polymorphic replacement or point-sensitive environment restoration.

The restored callback_moves suite passes all ten tests. The new owning-retention
control passes. The conditional scoped Once witness remains red. A direct arrow
initially appeared green because the existing owning parameter forced its array
capture to transfer. The strengthened witness reads the array after the call and
fails with transferred. Both reds are required: accepting a call by moving a
borrowed source does not satisfy the intended scoped semantics. Exact investigation
logs are stored under runtime/diagnostics/once-environment-admission/ in SQLite.

## Retention kernel checkpoint

callable_environment_demand.rs introduces a separate monotone retention graph.
Nodes use lexical BindingId values, preserving owner and declaration coordinates.
Edges propagate destination demand back to the source environment for declaration
moves, captures or forwarded arguments. The demand domain is Scoped, Owned and
Unknown. Missing endpoints become Unknown; unknown evidence cannot be downgraded
by later checked or retaining facts. A reverse-edge worklist propagates changes
without recursive dispatch. Each node can strengthen at most twice.

The original four tests fail before implementation. Six final controls pass for
returned captures and alias moves, recursive forwarding, distinct shadowed and
independent bindings, missing destinations, absorbing unknown evidence and a
10,000-node forwarding chain. The existing source index shadow/capture identity
control also passes. Whole-project formatting changes zero files and fingerprints.
Original red and green evidence is stored under runtime/diagnostics/once-environment-demand/
in benchmarks/records.sqlite with byte-exact readback and successful quick_check.

At the kernel checkpoint, this graph was not connected to source parameter resolution. Graph::checked must
only be supplied after the bridge accounts for all relevant source uses. Do not
claim that constructing a graph proves a body cannot retain an environment. The
next bridge should reuse generic_call_validation::effect_bodies::Index lexical
identities and capture requirements, with explicit treatment of every storage,
return, throw and call destination. Its API is currently private to that analysis;
expose a narrow source-fact adapter rather than duplicating name/range resolution.
Source Once reds remain red and owned retention remains intact. No editor consumer
or installed executable has been refreshed for this prerequisite.

## Source fact bridge in progress

once_environment_facts.rs reuses effect_bodies::Index source binding identities.
Nine focused controls pass for immediate calls and alias moves, direct and wrapped
returns, known forwarding, shadowed declarations, opaque callback destinations,
unchecked local factory results, transitive nested captures and aggregate storage.
The first five controls fail before implementation. Three additional boundary
witnesses expose false scoped inferences and are repaired before integration.

Known named callees propagate argument demands to the callee's parameter identities.
Opaque destinations and unproved spread positions remain unknown. Local aggregate
construction retains its existing owning storage requirement. Returned direct
wrappers retain their capture origins. The direct capture index is not treated as
proof of transitive nested environments. Uninvoked captured callback environments
also need checked lifetime admission before a scoped parameter contract is emitted.
Known ownership requirements stay owned while unresolved scoped capture admission
becomes unknown. Alias classification uses an iterative worklist.

The bridge now exposes a narrow parameter-demand adapter. Concrete named function
parameters use scoped Once only when their lexical demand is proven scoped.
Returning, storing or forwarding into unknown destinations preserves OwnedOnce.
Anonymous higher-order contracts and result annotations remain owning. This does
not prove complete transitive capture, spread or aggregate transport.

## Source integration verification in progress

Focused source/native witnesses now cover direct arrows with post-call reuse of
borrowed arrays, conditional consuming carriers, named forwarding and owned
retainers. Owned conditional families provide consuming invocation adapters and
adapt checked failure payloads at callback boundaries. The result adaptation helper
is shared by owning and borrowed families rather than duplicating failure mapping.

Higher-order parameter matching checks the caller-supplied Once input against the
concrete function parameter. An owning input can enter a scoped consumer. A borrowed
input cannot satisfy an owning retainer. The existing higher-order native witness
exposed the reversed comparison and now passes. A separate rejection control keeps
anonymous higher-order retaining boundaries from admitting borrowed environments.

Focused source integration is accepted after the bounded compiler checks, all eight
depth canaries, 334 editor tests, matched native/compiled-TypeScript oracle and durable
evidence import. See ../milestones/173-scoped-once-parameters.md. Full Gate173 remains open.
The matched workload now retains 133 observations, including scoped consumption
and explicit owned retention. No timing has been performed. Consumers remain at
the previous accepted source checkpoint until integration acceptance.
