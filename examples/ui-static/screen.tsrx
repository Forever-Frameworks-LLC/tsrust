import { View, Text, Button, Node } from 'std:ui';

export function Screen(): Node {
  return <View className="flex-col gap-4 p-4 bg-background">
    <Text className="text-xl font-bold">Tsrust UI</Text>
    <Text>{'One checked component.\nUnicode: café · العربية · 🦀'}</Text>
    <Button disabled className="p-2 rounded bg-primary text-primary-foreground">Static preview</Button>
  </View>;
}
