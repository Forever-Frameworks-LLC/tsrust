import { View, Text, Node, signal } from 'std:ui';
import { Alert, AlertTitle, AlertDescription, AlertAction } from '../components/ui/alert';
import { Button } from '../components/ui/button';

export function AlertActions(): Node {
  const retries = signal(0);
  const hidden = signal(false);
  return (
    <View className="gap-4">
      <Alert id="alert-action-example" hidden={hidden.get()} className="min-h-16">
        <AlertTitle>Connection interrupted</AlertTitle>
        <AlertDescription><Text id="alert-retries">Retry requests: {retries.get()}</Text></AlertDescription>
        <AlertAction><Button size="sm" className="text-xs py-1" onPress={() => { retries.set(retries.get() + 1); }}>Retry</Button></AlertAction>
      </Alert>
      <Button variant="outline" onPress={() => { hidden.set(!hidden.get()); }}>{hidden.get() ? 'Show notice' : 'Hide notice'}</Button>
    </View>
  );
}
