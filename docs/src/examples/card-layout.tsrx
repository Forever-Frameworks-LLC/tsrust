import { View, Text, Node, signal } from 'std:ui';
import { Card, CardHeader, CardTitle, CardDescription, CardAction, CardContent, CardFooter } from '../components/ui/card';
import { Button } from '../components/ui/button';

export function CardLayout(): Node {
  const shared = signal(false);
  return (
    <View className="grid grid-cols-1 md:grid-cols-2 gap-4 p-1">
      <Card>
        <CardHeader>
          <CardTitle><Text as="h3">Team workspace</Text></CardTitle>
          <CardDescription>Shared settings for your next project.</CardDescription>
          <CardAction><Button variant="outline" size="sm" onPress={() => { shared.set(!shared.get()); }}>Change access</Button></CardAction>
        </CardHeader>
        <CardContent><Text>Your project stays on this device until you choose to share it.</Text></CardContent>
        <CardFooter><Text className="text-muted-foreground">{shared.get() ? 'Shared workspace' : 'Private workspace'}</Text></CardFooter>
      </Card>
      <Card size="sm">
        <CardHeader>
          <CardTitle><Text as="h3">Compact card</Text></CardTitle>
          <CardDescription>The same parts with smaller spacing.</CardDescription>
        </CardHeader>
        <CardContent><Text>Long content wraps inside the available width, including when the documentation window narrows.</Text></CardContent>
      </Card>
    </View>
  );
}
