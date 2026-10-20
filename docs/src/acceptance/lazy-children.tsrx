import { View, Text, Button, Node, Child, LazyChildren, document } from 'std:ui';
import { strictEqual, throws, trackedResource, resourceId, releaseCount, liveCount, TrackedResource } from 'std:testing';

interface OptionalProps {
  children?: LazyChildren;
}

interface FactoryBase<T> {
  children: T;
}

interface RequiredProps extends FactoryBase<LazyChildren> {}

interface OnceProps extends FactoryBase < Once < LazyChildren >> {}

interface OrderedProps extends OptionalProps {
  id: string;
}

interface EagerProps {
  children: Child[];
}

interface TrackedProps {
  resource: TrackedResource;
}

function attribute(): string {
  console.log('attribute');
  return 'ordered';
}

function Ordered(props: OrderedProps): Node {
  console.log('parent');
  const render = props.children;
  return (<View id={props.id} children={render === undefined ? [] : render()}/>);
}

function OrderedChild(): Node {
  console.log('child');
  return (<Text>Ready</Text>);
}

function Eager(props: EagerProps): Node {
  console.log('eager-parent');
  return (<View children={props.children}/>);
}

function EagerChild(): Node {
  console.log('eager-child');
  return (<Text>Still eager</Text>);
}

function Ignore(props: OptionalProps): Node {
  console.log('ignored-parent');
  return (<View/>);
}

function ProbeChild(resource: ReadOnly<TrackedResource>): Node {
  console.log('probe-child');
  return (<Text>probe {resourceId(resource)}</Text>);
}

function Repeat(props: RequiredProps): Node {
  const render = props.children;
  const first = document(<View children={render()}/>, 'First');
  const second = document(<View children={render()}/>, 'Second');
  strictEqual(first.includes('probe'), true);
  strictEqual(second.includes('probe'), true);
  return (<View/>);
}

function OnceParent(props: OnceProps): Node {
  const render = props.children;
  return (<View children={render()}/>);
}

function TrackedChild(props: TrackedProps): Node {
  const resource = props.resource;
  return (<Button onPress={() => { console.log(resourceId(resource)); }}>Owned child</Button>);
}

function BrokenChild(): Node {
  console.log('failed-child');
  throw 'Expected child failure';
}

function onceResource(): int {
  const resource = trackedResource();
  const id = resourceId(resource);
  {
    const node = <OnceParent><TrackedChild resource={resource}/></OnceParent>;
    strictEqual(releaseCount(id), 0);
    strictEqual(document(node, 'Once').includes('Owned child'), true);
  }

  strictEqual(releaseCount(id), 1);
  return id;
}

function failedResource(): int {
  const resource = trackedResource();
  const id = resourceId(resource);
  throws(() => {
    const node = <OnceParent><TrackedChild resource={resource}/><BrokenChild/></OnceParent>;
  });
  strictEqual(releaseCount(id), 1);
  return id;
}

function main() {
  const baseline = liveCount();
  const ordered = <Ordered id={attribute()}><OrderedChild/></Ordered>;
  strictEqual(document(ordered, 'Order').includes('id="ordered"'), true);
  const eager = <Eager><EagerChild/></Eager>;
  strictEqual(document(eager, 'Eager').includes('Still eager'), true);
  const unused = trackedResource();
  const unusedId = resourceId(unused);
  const ignored = <Ignore>{ProbeChild(unused)}</Ignore>;
  strictEqual(releaseCount(unusedId), 1);
  const repeated = trackedResource();
  const repeatedId = resourceId(repeated);
  const repeat = <Repeat>{ProbeChild(repeated)}</Repeat>;
  strictEqual(releaseCount(repeatedId), 1);
  strictEqual(releaseCount(onceResource()), 1);
  strictEqual(releaseCount(failedResource()), 1);
  const empty = <Ordered id="empty"/>;
  strictEqual(document(empty, 'Empty').includes('id="empty"'), true);
  const explicit = <Ordered id="explicit" children={() => [<Text>Explicit</Text>]}/>;
  strictEqual(document(explicit, 'Explicit').includes('Explicit'), true);
  strictEqual(liveCount(), baseline);
  console.log('Lazy children passed');
}
