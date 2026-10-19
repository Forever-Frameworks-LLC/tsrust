import { View, Text, Node, signal } from 'std:ui';
import { Textarea } from '../components/ui/textarea';
import { Label } from '../components/ui/label';

export function MultilineDraft(): Node {
  const message = signal('First line\nSecond line');
  return (
    <View className="gap-3 p-1">
      <Label htmlFor="multiline-message">Release notes</Label>
      <Textarea id="multiline-message" name="notes" value={message} onChange={(event) => { message.set(event.target.value); }} className="max-h-48" />
      <Text role="status">UTF-8 bytes: {message.get().length}</Text>
      <Label htmlFor="multiline-readonly">Published notes</Label>
      <Textarea id="multiline-readonly" defaultValue="This copy is read-only. You can still select its text." readOnly={true} />
      <Label htmlFor="multiline-disabled">Archived draft</Label>
      <Textarea id="multiline-disabled" defaultValue="Editing is unavailable." disabled={true} />
    </View>
  );
}
