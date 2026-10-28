const assert = require('node:assert/strict');
const fs = require('node:fs');
const {execFileSync} = require('node:child_process');
const crypto = require('node:crypto');

function validatePull(pr, repository, expected) {
  assert.equal(pr.base.repo.full_name, repository, 'Wrong base repository');
  assert.equal(pr.head.repo?.full_name, repository, 'Promotion cannot originate in a fork');
  assert.equal(pr.base.ref, 'main', 'Promotion must target main');
  assert.equal(pr.head.ref, 'stag', 'Only stag can be promoted');
  assert.equal(pr.draft, false, 'Finish the draft before building releases');
  if (expected) {
    assert.equal(pr.number, expected.pull, 'Wrong promotion PR');
    assert.equal(pr.head.sha, expected.head, 'Staging changed; rebuild this promotion');
    if (!pr.merged) assert.equal(pr.base.sha, expected.base, 'Main changed; rebuild this promotion');
  }
  assert.ok(pr.merged || pr.state === 'open', 'Promotion was closed without merging');
}

async function absentTag(github, repo, tag) {
  try { await github.rest.git.getRef({...repo, ref: `tags/${tag}`}); }
  catch (error) { if (error.status === 404) return; throw error; }
  throw Error(`Release tag ${tag} already exists; update the compiler version before promotion`);
}

async function prepare({github, context, core}) {
  const repo = context.repo;
  const {data: pr} = await github.rest.pulls.get({...repo, pull_number: context.payload.pull_request.number});
  validatePull(pr, `${repo.owner}/${repo.repo}`);
  assert.equal(pr.head.sha, context.payload.pull_request.head.sha, 'PR changed before preflight');
  assert.equal(pr.base.sha, context.payload.pull_request.base.sha, 'Base changed before preflight');
  const tag = process.env.RELEASE_TAG;
  await absentTag(github, repo, tag);
  const {data: source} = await github.rest.git.getCommit({...repo, commit_sha: context.sha});
  assert.deepEqual(source.parents.map(parent => parent.sha), [pr.base.sha, pr.head.sha], 'Build must use the PR merge commit');
  const snapshot = {pull: pr.number, head: pr.head.sha, base: pr.base.sha, source: context.sha, tree: source.tree.sha, tag, run: String(context.runId)};
  fs.mkdirSync('target', {recursive: true});
  fs.writeFileSync('target/promotion.json', JSON.stringify(snapshot) + '\n');
  core.info(`Building ${tag} from ${snapshot.source}`);
}

function validateRun(run, snapshot, repository) {
  assert.equal(run.path, '.github/workflows/release.yml', 'Wrong build workflow');
  assert.equal(run.event, 'pull_request', 'Only a PR build may publish');
  assert.equal(run.conclusion, 'success', 'Release build did not pass');
  assert.equal(run.head_repository?.full_name, repository, 'Wrong build repository');
  assert.equal(run.head_branch, 'stag', 'Wrong build branch');
  assert.equal(run.head_sha, snapshot.head, 'Artifact head differs from the build');
  assert.equal(String(run.id), snapshot.run, 'Artifacts came from another run');
  assert.ok(run.pull_requests.some(pr => pr.number === snapshot.pull), 'Build is not associated with the promotion PR');
}

function validateArtifacts(directory, snapshot) {
  assert.match(snapshot.tag, /^v\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/);
  const targets = ['x86_64-unknown-linux-gnu', 'x86_64-pc-windows-msvc', 'aarch64-apple-darwin', 'x86_64-apple-darwin'];
  const names = fs.readdirSync(directory).sort();
  const archives = targets.map(target => `tsrust-${snapshot.tag}-${target}.tar.gz`);
  const extensions = names.filter(name => /^tsrust-.*\.vsix$/.test(name));
  assert.equal(extensions.length, 1, 'Expected one editor extension');
  assert.deepEqual(names, [...archives, ...extensions].sort(), 'Unexpected or missing release artifacts');
  for (const [index, file] of archives.entries()) {
    const manifest = JSON.parse(execFileSync('tar', ['-xOf', `${directory}/${file}`, `${file.slice(0, -7)}/manifest.json`], {encoding: 'utf8'}));
    assert.equal(manifest.sourceCommit, snapshot.source, 'Archive was built from another commit');
    assert.equal(manifest.sourceTreeDirty, false);
    assert.equal(manifest.distribution, 'release');
    assert.equal(manifest.version, snapshot.tag.slice(1));
    assert.equal(manifest.target, targets[index]);
  }
  const checksums = names.map(name => `${crypto.createHash('sha256').update(fs.readFileSync(`${directory}/${name}`)).digest('hex')}  ${name}`).join('\n') + '\n';
  fs.writeFileSync(`${directory}/SHA256SUMS`, checksums);
  return [...names, 'SHA256SUMS'];
}

async function mergeValidated({github, repo, snapshot}) {
  const {data: pr} = await github.rest.pulls.get({...repo, pull_number: snapshot.pull});
  validatePull(pr, `${repo.owner}/${repo.repo}`, snapshot);
  const {data: source} = await github.rest.git.getCommit({...repo, commit_sha: snapshot.source});
  assert.equal(source.tree.sha, snapshot.tree, 'Build tree provenance differs');
  assert.deepEqual(source.parents.map(parent => parent.sha), [snapshot.base, snapshot.head]);
  let sha = pr.merge_commit_sha;
  if (!pr.merged) {
    await absentTag(github, repo, snapshot.tag);
    const {data: main} = await github.rest.git.getRef({...repo, ref: 'heads/main'});
    assert.equal(main.object.sha, snapshot.base, 'Main advanced; rebuild this promotion');
    const {data: merged} = await github.rest.pulls.merge({...repo, pull_number: pr.number, sha: snapshot.head, merge_method: 'merge'});
    assert.equal(merged.merged, true, merged.message);
    sha = merged.sha;
  }
  const {data: commit} = await github.rest.git.getCommit({...repo, commit_sha: sha});
  assert.equal(commit.tree.sha, snapshot.tree, 'Merged tree differs; refusing to publish');
  return sha;
}

async function publish({github, context, core}) {
  const repo = context.repo;
  const snapshot = JSON.parse(fs.readFileSync('target/promotion.json'));
  const {data: run} = await github.rest.actions.getWorkflowRun({...repo, run_id: context.payload.workflow_run.id});
  validateRun(run, snapshot, `${repo.owner}/${repo.repo}`);
  const files = validateArtifacts('dist', snapshot);
  const sha = await mergeValidated({github, repo, snapshot});
  try {
    const {data: ref} = await github.rest.git.getRef({...repo, ref: `tags/${snapshot.tag}`});
    assert.equal(ref.object.sha, sha, 'Existing tag points to another commit');
  } catch (error) {
    if (error.status !== 404) throw error;
    await github.rest.git.createRef({...repo, ref: `refs/tags/${snapshot.tag}`, sha});
  }
  let release;
  try { release = (await github.rest.repos.getReleaseByTag({...repo, tag: snapshot.tag})).data; }
  catch (error) {
    if (error.status !== 404) throw error;
    release = (await github.rest.repos.createRelease({...repo, tag_name: snapshot.tag, target_commitish: sha, name: `Tsrust ${snapshot.tag}`, draft: true, prerelease: snapshot.tag.includes('-'), generate_release_notes: true, body: fs.readFileSync('.github/release-notes.md', 'utf8')})).data;
  }
  if (!release.draft) {
    assert.deepEqual(release.assets.map(asset => asset.name).sort(), files.sort(), 'Published asset set differs');
    for (const name of files) {
      const asset = release.assets.find(asset => asset.name === name);
      assert.equal(asset.digest, `sha256:${crypto.createHash('sha256').update(fs.readFileSync(`dist/${name}`)).digest('hex')}`, 'Published asset bytes differ');
    }
    core.info(`Already published ${release.html_url}`);
    return;
  }
  for (const asset of release.assets) await github.rest.repos.deleteReleaseAsset({...repo, asset_id: asset.id});
  for (const name of files) await github.rest.repos.uploadReleaseAsset({...repo, release_id: release.id, name, data: fs.readFileSync(`dist/${name}`), headers: {'content-type': 'application/octet-stream'}});
  const {data: published} = await github.rest.repos.updateRelease({...repo, release_id: release.id, draft: false});
  core.info(`Published ${published.html_url} without rebuilding`);
}
module.exports = {validatePull, validateRun, validateArtifacts, absentTag, prepare, mergeValidated, publish};
