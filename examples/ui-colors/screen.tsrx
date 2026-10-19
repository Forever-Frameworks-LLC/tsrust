import { View, Text, Button, Node, withTheme } from 'std:ui';
import config from './tsrust.config.json';

export function Screen(): Node {
  return withTheme((
    <View role="region" aria-label="Theme colors" className="p-4 gap-4">
      <View data-slot="panel" className="p-4 gap-2 rounded-lg border border-input/50 bg-card/50 ring-1 ring-foreground/10">
        <Text className="text-xl font-bold">Transparent surfaces</Text>
        <Text>Checked colors keep the original theme data.</Text>
        <View data-slot="primary" className="h-8 bg-primary/50" />
        <View data-slot="transparent" className="h-8 bg-transparent ring-1 ring-border" />
        <View data-slot="alias" className="h-8 bg-accent" />
        <View data-slot="child-ring" className="h-8 ring-1 bg-background" />
        <Button className="rounded bg-foreground text-background hover:bg-foreground/90">Continue</Button>
      </View>
    </View>
  ), config.ui.theme);
}
