import { View, Text, Node, Link } from 'std:ui';
import { Alert, AlertTitle, AlertDescription, AlertAction } from '../components/ui/alert';

export function AlertLayouts(): Node {
  return (
    <View className="gap-4">
      <Alert className="border-primary bg-primary/5"><AlertTitle>Keep the source local</AlertTitle><AlertDescription>Installed components are ordinary project files. Adjust their layout and tokens alongside your application.</AlertDescription></Alert>
      <Alert className="has-data-[slot=alert-action]:pr-2.5">
        <AlertTitle>More room for actions</AlertTitle>
        <AlertDescription>Place long actions in the normal flow so they remain readable on narrow screens.</AlertDescription>
        <AlertAction className="relative top-auto right-auto flex-row flex-wrap gap-3 pt-2">
          <Link href="/docs/ui/foundations/installation" className="text-primary py-2 focus:ring">Install local components</Link>
          <Text className="text-xs text-muted-foreground">No registry request.</Text>
        </AlertAction>
      </Alert>
    </View>
  );
}
