import { Text, Button, View, Node, signal, withTheme } from 'std:ui';
import { Card, CardHeader, CardTitle, CardDescription, CardAction, CardContent, CardFooter } from './components/ui/card';
import config from '../tsrust.config.json';

export function Screen(): Node {
  const presses = signal(0);
  return withTheme((
    <View className="p-4 gap-4">
      <Text data-slot="press-count">Presses: {presses}</Text>
      <Card id="default-card" role="region" aria-label="Project settings" className="w-64">
        <CardHeader>
          <CardTitle><Text as="h2">Project settings</Text></CardTitle>
          <CardDescription>Editable local components</CardDescription>
          <CardAction><Button>More</Button></CardAction>
        </CardHeader>
        <CardContent><Text>Default card body</Text></CardContent>
        <CardFooter><Button>Save</Button></CardFooter>
      </Card>
      <Card id="small-card" size="sm" className="w-64">
        <CardHeader><CardTitle>Small card</CardTitle></CardHeader>
        <CardContent>Small body</CardContent>
      </Card>
      <Card id="custom-card" className="w-64 [--card-spacing:--spacing(6)] rounded-none">
        <CardHeader className="border-b"><CardTitle>Custom spacing</CardTitle><CardDescription>Source override</CardDescription></CardHeader>
        <CardContent className="font-mono">Monospace body</CardContent>
        <CardFooter>Footer</CardFooter>
      </Card>
      <Card id="clip-card" className="w-16 h-16 p-0 rounded-xl [--card-spacing:--spacing(0)]">
        <View className="w-64 shrink-0 h-32 bg-primary text-primary-foreground"><Button onPress={() => { presses.set(presses.get() + 1); }}>Clipped action</Button></View>
      </Card>
      <View id="missing-variable" className="p-4 p-(--absent)"><Text>Missing spacing is zero</Text></View>
    </View>
  ), config.ui.theme);
}
