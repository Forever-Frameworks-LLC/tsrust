import { View, Node } from 'std:ui';
import { Alert, AlertTitle, AlertDescription } from '../components/ui/alert';

export function AlertVariants(): Node {
  return (
    <View className="gap-4">
      <Alert id="alert-default"><AlertTitle>Your workspace is ready</AlertTitle><AlertDescription>Start with an entrypoint, then add the components you need.</AlertDescription></Alert>
      <Alert id="alert-destructive" variant="destructive"><AlertTitle>Connection failed</AlertTitle><AlertDescription>Check the server address before trying again.</AlertDescription></Alert>
    </View>
  );
}
