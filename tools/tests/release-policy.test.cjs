const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const {execFileSync} = require('node:child_process');
const {validatePull, validateRun, validateArtifacts, absentTag, mergeValidated, publish} = require('../../.github/scripts/release-policy.cjs');
const repository = 'owner/project';
const repo = {owner: 'owner', repo: 'project'};
const snapshot = {pull: 7, head: 'staged', base: 'main', source: 'tested-merge', tree: 'tree', tag: 'v1.2.3', run: '42'};
const pull = () => ({number: 7, state: 'open', draft: false, merged: false,
  base: {ref: 'main', sha: 'main', repo: {full_name: repository}},
  head: {ref: 'stag', sha: 'staged', repo: {full_name: repository}}, merge_commit_sha: 'merged'});
const run = () => ({path: '.github/workflows/release.yml', event: 'pull_request', conclusion: 'success',
  head_repository: {full_name: repository}, head_branch: 'stag', head_sha: 'staged', id: 42, pull_requests: [{number: 7}]});
const notFound = () => { throw Object.assign(Error('Not found'), {status: 404}); };

test('only the unchanged, same-repository staging PR can promote', () => {
  validatePull(pull(), repository, snapshot);
  for (const mutate of [pr => pr.head.repo.full_name = 'fork/project', pr => pr.base.repo.full_name = 'other/project',
    pr => pr.head.ref = 'feature', pr => pr.base.ref = 'stag', pr => pr.draft = true,
    pr => pr.head.sha = 'new', pr => pr.base.sha = 'new', pr => pr.number++, pr => pr.state = 'closed']) {
    const pr = pull(); mutate(pr);
    assert.throws(() => validatePull(pr, repository, snapshot));
  }
  const merged = {...pull(), merged: true, state: 'closed'};
  merged.base.sha = 'merged';
  validatePull(merged, repository, snapshot);
});

test('failed, manual, unrelated or substituted builds cannot publish', () => {
  validateRun(run(), snapshot, repository);
  for (const mutation of [{path: '.github/workflows/other.yml'}, {event: 'workflow_dispatch'}, {conclusion: 'failure'},
    {head_repository: {full_name: 'fork/project'}}, {head_branch: 'feature'}, {head_sha: 'new'}, {id: 43}, {pull_requests: []}]) {
    assert.throws(() => validateRun({...run(), ...mutation}, snapshot, repository));
  }
});

test('tag lookup only treats a genuine 404 as available', async () => {
  await absentTag({rest: {git: {getRef: notFound}}}, repo, snapshot.tag);
  await assert.rejects(absentTag({rest: {git: {getRef: async () => ({})}}}, repo, snapshot.tag), /already exists/);
  await assert.rejects(absentTag({rest: {git: {getRef: async () => { throw Object.assign(Error('Forbidden'), {status: 403}); }}}}, repo, snapshot.tag), /Forbidden/);
});

function merger(options = {}) {
  const calls = [];
  const pr = {...pull(), ...options.pr};
  const github = {rest: {
    pulls: {get: async () => ({data: pr}), merge: async args => {
      calls.push(args); return {data: {merged: !options.reject, sha: 'merged', message: 'Checks pending'}};
    }},
    git: {
      getRef: async ({ref}) => ref === 'heads/main' ? {data: {object: {sha: options.main || 'main'}}} : notFound(),
      getCommit: async ({commit_sha}) => ({data: {tree: {sha: commit_sha === 'merged' && options.wrongTree ? 'other' : 'tree'},
        parents: [{sha: options.parent || 'main'}, {sha: 'staged'}]}})
    }
  }};
  return {github, calls};
}

test('merge is pinned to the tested head, and an already-merged retry does not merge twice', async () => {
  const first = merger();
  assert.equal(await mergeValidated({...first, repo, snapshot}), 'merged');
  assert.deepEqual(first.calls, [{...repo, pull_number: 7, sha: 'staged', merge_method: 'merge'}]);
  const retry = merger({pr: {merged: true, state: 'closed'}});
  assert.equal(await mergeValidated({...retry, repo, snapshot}), 'merged');
  assert.equal(retry.calls.length, 0);
});

test('stale main, substituted parents, rejected merges and changed merged trees stop promotion', async () => {
  for (const options of [{main: 'advanced'}, {parent: 'other'}, {reject: true}, {wrongTree: true}]) {
    const fixture = merger(options);
    await assert.rejects(mergeValidated({...fixture, repo, snapshot}));
    if (options.main || options.parent) assert.equal(fixture.calls.length, 0);
  }
});

async function artifacts(body) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'tsrust-promotion-'));
  const dist = path.join(root, 'dist');
  fs.mkdirSync(dist);
  const targets = ['x86_64-unknown-linux-gnu', 'x86_64-pc-windows-msvc', 'aarch64-apple-darwin', 'x86_64-apple-darwin'];
  function archive(target, overrides = {}) {
    const name = `tsrust-${snapshot.tag}-${target}`;
    const source = path.join(root, name);
    fs.mkdirSync(source, {recursive: true});
    fs.writeFileSync(path.join(source, 'manifest.json'), JSON.stringify({sourceCommit: snapshot.source, sourceTreeDirty: false,
      distribution: 'release', version: '1.2.3', target, ...overrides}));
    execFileSync('tar', ['-czf', path.join(dist, `${name}.tar.gz`), '-C', root, name]);
  }
  try {
    targets.forEach(target => archive(target));
    fs.writeFileSync(path.join(dist, 'tsrust-0.2.0.vsix'), 'extension');
    await body({dist, archive, target: targets[0]});
  } finally { fs.rmSync(root, {recursive: true, force: true}); }
}

test('four platform archives and the extension produce checksums of their original bytes', () => artifacts(({dist}) => {
  const files = validateArtifacts(dist, snapshot);
  assert.equal(files.length, 6);
  const sums = fs.readFileSync(path.join(dist, 'SHA256SUMS'), 'utf8').trim().split('\n');
  assert.equal(sums.length, 5);
  for (const line of sums) assert.match(line, /^[a-f0-9]{64}  tsrust-/);
}));

test('missing, extra, dirty or wrong-source release artifacts are rejected before merge', () => artifacts(({dist, archive, target}) => {
  for (const overrides of [{sourceCommit: 'other'}, {sourceTreeDirty: true}, {distribution: 'development'}, {version: '9.0.0'}, {target: 'other'}]) {
    archive(target, overrides);
    assert.throws(() => validateArtifacts(dist, snapshot));
  }
  archive(target);
  fs.writeFileSync(path.join(dist, 'unexpected'), 'extra');
  assert.throws(() => validateArtifacts(dist, snapshot));
  fs.unlinkSync(path.join(dist, 'unexpected'));
  fs.unlinkSync(path.join(dist, 'tsrust-0.2.0.vsix'));
  assert.throws(() => validateArtifacts(dist, snapshot));
}));


test('publication uploads validated bytes after merge and safely resumes a partial draft', async () => artifacts(async ({dist}) => {
  const cwd = process.cwd();
  process.chdir(path.dirname(dist));
  try {
    fs.mkdirSync('target');
    fs.mkdirSync('.github');
    fs.writeFileSync('target/promotion.json', JSON.stringify(snapshot));
    fs.writeFileSync('.github/release-notes.md', 'Release notes');
    const fixture = merger();
    const events = [];
    const github = fixture.github;
    github.rest.actions = {getWorkflowRun: async () => ({data: run()})};
    github.rest.git.createRef = async args => { events.push(['tag', args.sha]); };
    github.rest.repos = {
      getReleaseByTag: async () => ({data: {id: 1, draft: true, assets: [{id: 9, name: 'partial.tar.gz'}]}}),
      deleteReleaseAsset: async ({asset_id}) => { events.push(['delete-draft-asset', asset_id]); },
      uploadReleaseAsset: async ({name, data}) => {
        assert.deepEqual(data, fs.readFileSync(path.join(dist, name)));
        assert.equal(fixture.calls.length, 1);
        events.push(['upload', name]);
      },
      updateRelease: async ({draft}) => {
        assert.equal(draft, false);
        assert.equal(events.filter(event => event[0] === 'upload').length, 6);
        events.push(['publish']);
        return {data: {html_url: 'https://example.test/release'}};
      }
    };
    await publish({github, context: {repo, payload: {workflow_run: {id: 42}}}, core: {info() {}}});
    assert.deepEqual(events[0], ['tag', 'merged']);
    assert.deepEqual(events[1], ['delete-draft-asset', 9]);
    assert.deepEqual(events.at(-1), ['publish']);
    // Reruns download the original artifacts, without the generated checksum file.
    fs.unlinkSync(path.join(dist, 'SHA256SUMS'));
    github.rest.actions.getWorkflowRun = async () => ({data: {...run(), conclusion: 'failure'}});
    const count = events.length;
    await assert.rejects(publish({github, context: {repo, payload: {workflow_run: {id: 42}}}, core: {info() {}}}));
    assert.equal(events.length, count);
  } finally { process.chdir(cwd); }
}));
