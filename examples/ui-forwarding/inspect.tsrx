import { Screen } from './screen';
import { View, ViewProps, Text, Node, document } from 'std:ui';
import { strictEqual, trackedResource, TrackedResource, resourceId, releaseCount, closeResource } from 'std:testing';

interface ForwardProps extends ViewProps {
  extra: TrackedResource;
}

function Forward(props: ForwardProps): Node {
  return <View {...props} />;
}

interface OwnerProps {
  owner: TrackedResource;
  title?: string;
}

interface ExtraProps extends OwnerProps {
  extra: TrackedResource;
}

function Consume(props: OwnerProps): int {
  strictEqual(props.title === undefined, true);
  closeResource(props.owner);
  return 1;
}

function main() {
  const html = document(Screen(), 'Forwarding');
  strictEqual(html.includes('data-size="sm"'), true);
  strictEqual(html.includes(':where(:has([data-slot="footer"]))'), true);
  strictEqual(html.includes(':where(.group\\/panel[data-size="sm"] *)'), true);
  const first = trackedResource();
  const firstId = resourceId(first);
  {
    const node = <Forward extra={first}><Text>Owned child</Text></Forward>;
    strictEqual(releaseCount(firstId), 1);
    strictEqual(document(node, 'Owned').includes('Owned child'), true);
  }

  strictEqual(releaseCount(firstId), 1);
  const owner = trackedResource();
  const extra = trackedResource();
  const ownerId = resourceId(owner);
  const extraId = resourceId(extra);
  const props: ExtraProps = {
    owner: owner, extra: extra
  };
  strictEqual(<Consume {...props}/>, 1);
  strictEqual(releaseCount(ownerId), 1);
  strictEqual(releaseCount(extraId), 1);
  console.log('UI forwarding passed');
}
