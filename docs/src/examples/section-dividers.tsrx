import { View, Text, Node } from 'std:ui';
import { Separator } from '../components/ui/separator';

export function SectionDividers(): Node {
  return (
    <View className="gap-4 p-1">
      <View className="gap-1"><Text className="font-semibold">Workspace</Text><Text className="text-muted-foreground">Separate related sections without adding a heading.</Text></View>
      <Separator />
      <View className="flex-row gap-4 h-8 items-center"><Text>Profile</Text><Separator orientation="vertical" /><Text>Settings</Text></View>
    </View>
  );
}
