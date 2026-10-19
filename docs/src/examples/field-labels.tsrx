import { View, Node } from 'std:ui';
import { Input } from '../components/ui/input';
import { Label } from '../components/ui/label';

export function FieldLabels(): Node {
  return (
    <View className="gap-4 p-1">
      <View className="gap-2"><Label htmlFor="field-labels-explicit">Explicit association</Label><Input id="field-labels-explicit" placeholder="Click the label to focus" /></View>
      <Label className="flex-col items-start gap-2">Enclosed control<Input placeholder="This field has an enclosing label" /></Label>
    </View>
  );
}
