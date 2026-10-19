import { View, Text, Button, Node, signal } from 'std:ui';
import { Kbd, KbdGroup } from '../components/ui/kbd';
import { Separator } from '../components/ui/separator';

export function Page(): Node {
  const count = signal(0);
  return (
    <View className="p-4 gap-4 bg-background">
   <Text as="h1">Keyboard content and separators</Text>
   <KbdGroup><Kbd>Ctrl</Kbd>+<Kbd>K</Kbd></KbdGroup>
   <Text>Keycaps display text; application code owns keyboard behavior.</Text>
   <Separator id="horizontal"/>
   <View className="flex-row items-center h-12 gap-4">
    <Kbd id="before">A</Kbd>
    <Separator id="vertical" orientation="vertical"/>
    <Kbd id="after">B</Kbd>
   </View>
   <Button aria-label="Count" onPress={()=>{count.set(count.get()+1);}}>Count <Kbd id="live">{count.get()+1}</Kbd></Button>
   <View data-slot="tooltip-content" className="bg-primary p-2">
    <Kbd id="nested">Scoped</Kbd>
   </View>
  </View>
  );
}
