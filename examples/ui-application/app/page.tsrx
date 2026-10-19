import { View, Text, signal, Node } from 'std:ui';
import { Button } from '../components/ui/button';
import { Card } from '../components/ui/card';

export function Page(): Node {
  const count = signal(0);
  return (
    <View className="p-4 gap-4">
   <Card>
    <Text className="text-xl">Count: {count}</Text>
    <Button disabled={count.get() < 5} onPress={()=>{count.set(count.get()+100);}}>Disabled</Button>
    <Button variant="outline" size="lg" onPress={()=>{count.set(count.get()+1);}} className="rounded-lg focus:bg-secondary">Increment</Button>
   </Card>
  </View>
  );
}
