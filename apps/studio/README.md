# Tsrust Studio

Native developer-tool application under active implementation. Created with
`tsrust new apps/studio --ui`; application source remains `.tsr`/`.tsrx`.

Run `tsrust dev --verbose` in this directory for per-stage rebuild/cache diagnostics.
On macOS, UI dev builds have a temporary native app identity; no signing or release
packaging runs. Failed builds keep the previous app running. Successful rebuilds
restart the child and currently reset this scratch buffer. The current independent preview is an unsaved
scratch editor with working Find/Replace and light/dark theme controls. Closing
it discards the scratch buffer. Project editing/saving and recovery are pending;
do not use this development preview for irreplaceable work.

Run `tsrust test tests --profile` for the revision/proposal acceptance suite.
The separate native-AI admission lives in `experiments/studio-ai`; it is not a
Studio runtime dependency or a qualified model distribution.

Full required scope and current evidence:
[plan](../../docs/plans/tsrust-studio.md),
[capability audit](../../docs/plans/tsrust-studio-audit.md).
Signing, packaging and publication follow full acceptance and owner authorization.
