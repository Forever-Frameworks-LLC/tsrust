import { View, Text, Button, Node, signal, withTheme } from 'std:ui';
import config from '../../tsrust.config.json';
import { Input } from '../components/ui/input';
import { Textarea } from '../components/ui/textarea';
import { Label } from '../components/ui/label';

export function Page(): Node {
  const name = signal('Ada');
  const notes = signal('First');
  const locked = signal(false);
  const edits = signal(0);
  const last = signal('');
  return (
    <View className="p-6 gap-4 max-w-96">
      <Text as="h1" className="text-xl">Editable profile</Text>
      <Label htmlFor="name">Name</Label>
      <Input id="name" name="name" value={name.get()} placeholder="Your name" aria-describedby="name-help" required readOnly={locked.get()} onChange={(event) => { name.set(event.target.value); edits.set(edits.get() + 1); last.set(event.target.name + ':' + event.currentTarget.id); }}/>
      <Text id="name-help">Use your full name.</Text>
      <Label htmlFor="notes">Notes</Label>
      <Textarea id="notes" name="notes" value={notes} placeholder="Notes" onChange={(event) => { notes.set(event.currentTarget.value); }}/>
      <Input aria-label="Derived name" value={name.get() + '!'} readOnly/>
      <Input aria-label="Exact integer" value={9007199254740993} readOnly/>
      <Input aria-label="Uncontrolled" defaultValue="Initial"/>
      <Input aria-label="Disabled" defaultValue="Fixed" disabled/>
      <Input aria-label="Invalid" defaultValue="Review" aria-invalid/>
      <Button onPress={() => { locked.set(!locked.get()); }}>Toggle read-only</Button>
      <Button onPress={() => { name.set('Reset'); notes.set('One\nTwo'); }}>Reset</Button>
      {withTheme(<View className="p-2"><Input aria-label="Dark invalid" defaultValue="Review" aria-invalid disabled/></View>, config.ui.theme, 'dark')}
      <Text data-slot="result">{name} / {notes} / {edits} / {locked} / {last}</Text>
    </View>
  );
}
