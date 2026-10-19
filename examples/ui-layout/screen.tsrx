import { View, Node, withTheme } from 'std:ui';
import config from './tsrust.config.json';

export function Screen(dark: boolean): Node {
  return withTheme((
    <View role="region" aria-label="Layout witness" data-slot="grid" className="grid grid-cols-2 gap-4 p-4">
      <View data-slot="a" className="min-w-0 h-8 bg-primary" />
      <View data-slot="b" className="col-start-2 row-start-1 row-span-2 bg-accent" />
      <View data-slot="c" className="col-start-1 row-start-2 h-8 bg-secondary" />
    </View>
  ), config.ui.theme, dark ? 'dark' : 'light');
}
