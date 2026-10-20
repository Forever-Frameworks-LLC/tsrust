import { View, Text, Node } from 'std:ui';

function mark(value: string): string {
  console.log(value);
  return value;
}

export function Order(): Node {
  return (<View className={mark('p-2')} {...{id: 'source-order'}} data-variant={mark('default')}><Text>{mark('child')}</Text></View>);
}
