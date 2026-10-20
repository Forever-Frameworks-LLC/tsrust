import { View, Text, Node, Svg, Path, signal, withTheme, ThemeConfig } from 'std:ui';
import { Kbd, KbdGroup } from '../components/ui/kbd-v2';
import { Button } from '../components/ui/button-v3';
import config from '../../tsrust.config.json';

export function KeyboardShortcuts(): Node {
  return (
    <View id="keyboard-shortcuts" className="gap-4 p-1">
      <View className="flex-row flex-wrap items-center gap-3"><Text>Command palette</Text><KbdGroup><Kbd>Ctrl</Kbd><Text>+</Text><Kbd>K</Kbd></KbdGroup></View>
      <View className="flex-row flex-wrap items-center gap-3"><Text>Close</Text><Kbd>Esc</Kbd></View>
    </View>
  );
}

export function KeyboardCustomization(): Node {
  const accent = signal(false);
  return (
    <View id="keyboard-customization" className="gap-4">
      <View className="flex-row flex-wrap items-center gap-3">
        <Text>Save changes</Text>
        <KbdGroup id="keyboard-live-group" className={accent.get() ? 'gap-2' : 'gap-1'}>
          <Kbd id="keyboard-live-key" className={accent.get() ? 'h-8 px-3 rounded-lg bg-primary text-primary-foreground' : 'h-5 px-1 rounded-sm bg-muted text-muted-foreground'}>Ctrl</Kbd>
          <Text>+</Text><Kbd>S</Kbd>
        </KbdGroup>
      </View>
      <Button id="keyboard-change-style" variant="outline" onPress={() => { accent.set(!accent.get()); }}>Change keycap style</Button>
    </View>
  );
}

export function KeyboardIconsAndTheme(): Node {
  const mode = signal('light');
  const theme: ThemeConfig = {
    schemaVersion: 1, light: {
      ...clone(config.ui.theme.light)
    }, dark: {
      ...clone(config.ui.theme.dark)
    }, spacing: {
      sm: 10, md: 20, lg: 30
    }, radius: {
      ...clone(config.ui.theme.radius), sm: 8
    }, typography: {
      base: 20, xl: 28
    }
  };
  return withTheme(
    <View id="keyboard-theme" className="gap-4 rounded-lg bg-background text-foreground p-4">
      <View className="flex-row flex-wrap items-center gap-3">
        <Kbd id="keyboard-themed-key" className="font-mono"><Svg id="keyboard-default-icon" fill="none" stroke="currentColor" strokeWidth={2}><Path d="M4 12L9 17L20 6"/></Svg>Enter</Kbd>
        <Kbd className="h-auto p-2"><Svg id="keyboard-sized-icon" className="size-6" fill="none" stroke="currentColor" strokeWidth={2}><Path d="M4 4H20V20H4Z"/></Svg>Space</Kbd>
      </View>
      <View data-slot="tooltip-content" className="rounded bg-foreground text-background p-3 flex-row items-center gap-2"><Text className="text-sm">Open search</Text><Kbd id="keyboard-ancestor-key">/</Kbd></View>
      <Button id="keyboard-change-theme" variant="outline" onPress={() => { mode.set(mode.get() === 'light' ? 'dark' : 'light'); }}>Toggle keycap theme</Button>
    </View>, theme, mode);
}
