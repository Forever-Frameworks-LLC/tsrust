import { View, Text, Node, signal, withTheme } from 'std:ui';
import { Alert, AlertTitle, AlertDescription } from '../components/ui/alert';
import { Button } from '../components/ui/button';
import config from '../../tsrust.config.json';

export function AlertTheme(): Node {
  const mode = signal('light');
  const theme = clone(config.ui.theme);
  theme.light.card = '#fff7ed';
  theme.light.cardForeground = '#431407';
  theme.dark.card = '#1c1917';
  theme.dark.cardForeground = '#ffedd5';
  return withTheme(
    <View id="alert-theme-scope" className="p-4 gap-4 rounded-lg bg-card text-card-foreground">
      <Alert><AlertTitle>A palette of your own</AlertTitle><AlertDescription>Card, foreground and border tokens follow this scope.</AlertDescription></Alert>
      <Text id="alert-local-mode">Local mode: {mode.get()}</Text>
      <Button variant="outline" onPress={() => { mode.set(mode.get() === 'light' ? 'dark' : 'light'); }}>Toggle example theme</Button>
    </View>, theme, mode);
}
