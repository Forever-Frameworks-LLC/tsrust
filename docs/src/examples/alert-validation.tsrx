import { View, Node, signal } from 'std:ui';
import { Alert, AlertTitle, AlertDescription } from '../components/ui/alert';
import { Input } from '../components/ui/input';
import { Label } from '../components/ui/label';

export function AlertValidation(): Node {
  const name = signal('');
  return (
    <View className="gap-3">
      <Label htmlFor="alert-project-name">Project name</Label>
      <Input id="alert-project-name" value={name} onChange={(event) => { name.set(event.target.value); }} />
      <Alert id="alert-validation-message" variant="destructive" role="status" hidden={name.get().trim().length !== 0}>
        <AlertTitle>Give your project a name</AlertTitle>
        <AlertDescription>The name must contain more than whitespace.</AlertDescription>
      </Alert>
    </View>
  );
}
