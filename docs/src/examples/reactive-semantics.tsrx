import { View, Text, Input, Button, Svg, Path, Node, signal } from 'std:ui';
import { Button as LocalButton } from '../components/ui/button';
import { Badge } from '../components/ui/badge';

export function ReactiveSemanticsExample(): Node {
  const changed = signal(false);
  return (
    <View className="gap-4">
      <Input id="metadata-draft" defaultValue="Kept draft" aria-label={changed.get() ? 'Renamed draft' : 'Draft'} aria-describedby={changed.get() ? 'metadata-error' : 'metadata-hint'} aria-invalid={changed.get() ? true : undefined} data-size={changed.get() ? 'lg' : 'sm'} className="rounded border px-3 py-2 aria-invalid:border-destructive"/>
      <Text id="metadata-hint" className="text-sm text-muted-foreground">Keep this draft while changing metadata.</Text>
      <Text id="metadata-error" className="text-sm text-destructive">A validation message chosen by application code.</Text>
      <LocalButton id="metadata-toggle" aria-label={changed.get() ? 'Restore metadata' : 'Change metadata'} onPress={() => { changed.set(!changed.get()); }}>Change metadata</LocalButton>
      <Badge id="metadata-badge" aria-label={changed.get() ? 'Updated status' : 'Initial status'} render={(props) => <Button {...props}/>}>Status action</Badge>
      <View id="metadata-decoration" aria-hidden={changed.get()}><Text>Decorative detail</Text></View>
      <View id="metadata-size" data-size={changed.get() ? 'lg' : 'sm'} className="rounded border data-[size=sm]:p-2 data-[size=lg]:p-6"><Text>Size metadata selects spacing.</Text></View>
      <Svg id="metadata-icon" aria-label={changed.get() ? 'Saved' : undefined} viewBox="0 0 24 24" className="w-6 h-6 text-primary" fill="none" stroke="currentColor" strokeWidth={2}><Path d="M4 12L9 17L20 6"/></Svg>
    </View>
  );
}
