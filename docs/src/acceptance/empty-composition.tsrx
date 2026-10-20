import { View, Text, document } from 'std:ui';
import { trackedResource, resourceId, releaseCount, strictEqual } from 'std:testing';
import { Empty } from '../components/ui/empty';

export function verifyEmptyComposition() {
  const resource = trackedResource();
  const id = resourceId(resource);
  {
    const node = <Empty className={{ evaluate: () => { strictEqual(releaseCount(resourceId(resource)), 0); return 'p-6'; } }} render={(props) => (<View {...props} role="region" aria-label="Empty slot"/>)}><Text>Forwarded children</Text></Empty>;
    const html = document(node, 'Composition');
    strictEqual(html.includes('Forwarded children'), true);
    strictEqual(html.includes('aria-label="Empty slot"'), true);
    strictEqual(releaseCount(id), 0);
  }

  strictEqual(releaseCount(id), 1);
}
