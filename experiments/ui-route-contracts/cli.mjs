import {readFileSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
import {discover, matchRoute, buildPath} from './routes.mjs';

try {
  const [command, root, declarationsPath, ...extra] = process.argv.slice(2);
  let output;
  if (command === 'demo' && root === undefined) {
    const fixture = fileURLToPath(new URL('./fixtures/editable-list', import.meta.url));
    const declarations = JSON.parse(readFileSync(path.join(fixture, 'handlers.json'), 'utf8'));
    const result = discover(fixture, declarations);
    if (!result.manifest) throw new Error(JSON.stringify(result.diagnostics));
    output = {...result, matches: ['/login', '/health', '/items', '/items/3', '/items/new', '/'].map(url => ({url, match: matchRoute(result.manifest, url)})),
      builtPath: buildPath(result.manifest, 'app/items/[id]', {id:'a b'}),
      collision: discover(fixture, {...declarations, 'app/items/route.tsr': [{method:'GET', path:'/'}]})};
  } else if (command === 'inspect' && root && declarationsPath && !extra.length) {
    output = discover(path.resolve(root), JSON.parse(readFileSync(declarationsPath, 'utf8')));
    if (!output.manifest) process.exitCode = 1;
  } else throw new Error('Usage: cli.mjs demo | inspect <project-root> <handlers.json>');
  console.log(JSON.stringify(output, null, 2));
} catch (error) {
  console.error(error.message);
  process.exitCode = 1;
}
