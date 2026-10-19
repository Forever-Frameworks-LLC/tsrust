import { View, Text, Node, signal } from 'std:ui';
import { Input } from '../components/ui/input';
import { Label } from '../components/ui/label';

export function InputStates(): Node {
  const name = signal('Ada');
  return (
    <View className="gap-4 p-1">
      <View className="gap-2">
        <Label htmlFor="input-states-name">Display name</Label>
        <Input id="input-states-name" name="displayName" value={name} onChange={(event) => { name.set(event.target.value); }} />
        <Text role="status">Hello, {name}.</Text>
      </View>
      <View className="gap-2"><Label htmlFor="input-states-draft">Uncontrolled draft</Label><Input id="input-states-draft" defaultValue="Edit me; source switching keeps this text." /></View>
      <View className="gap-2"><Label htmlFor="input-states-readonly">Read-only reference</Label><Input id="input-states-readonly" defaultValue="REF-2026" readOnly={true} /></View>
      <View className="gap-2"><Label htmlFor="input-states-disabled">Unavailable field</Label><Input id="input-states-disabled" defaultValue="Not editable" disabled={true} /></View>
      <View className="gap-2"><Label htmlFor="input-states-invalid">Project slug</Label><Input id="input-states-invalid" defaultValue="two words" aria-invalid={true} aria-describedby="input-states-error" /><Text id="input-states-error" className="text-sm text-destructive">Use a single word without spaces.</Text></View>
    </View>
  );
}
