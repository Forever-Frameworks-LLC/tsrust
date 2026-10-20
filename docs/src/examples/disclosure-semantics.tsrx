import { View, Text, Input, Button, Node, signal, useId } from 'std:ui';
import { Button as LocalButton } from '../components/ui/button-v3';

interface DetailsProps {
  title: string;
}

function Details(props: DetailsProps): Node {
  const open = signal(false);
  const panelId = useId();
  const controlsId = clone(panelId);
  return (
    <View className="gap-3 rounded-lg border p-4">
      <LocalButton data-slot="disclosure-trigger" aria-expanded={open.get()} aria-controls={open.get() ? clone(controlsId) : undefined} onPress={() => { open.set(!open.get()); }} className="justify-between">{props.title}</LocalButton>
      <View id={panelId} data-slot="disclosure-panel" hidden={!open.get()} className="gap-2">
        <Text className="text-sm text-muted-foreground">Closing keeps this draft. Each panel has independent state.</Text>
        <Input defaultValue="Untitled project" aria-label="Project draft" className="w-full rounded border px-3 py-2"/>
      </View>
    </View>
  );
}

export function DisclosureSemanticsExample(): Node {
  return (
    <View id="disclosure-example" className="gap-4">
      <Details title="Project settings"/>
      <Details title="Workspace settings"/>
      <Button id="disclosure-disabled" disabled aria-expanded={false} className="rounded border px-3 py-2 opacity-50">Unavailable settings</Button>
    </View>
  );
}
