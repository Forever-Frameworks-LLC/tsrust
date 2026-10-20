import { Text, Node, signal } from 'std:ui';
import { Card, CardHeader, CardTitle, CardDescription, CardContent, CardFooter } from '../components/ui/card';
import { Button } from '../components/ui/button';
import { Input } from '../components/ui/input';
import { Label } from '../components/ui/label';

export function CardProject(): Node {
  const name = signal('Fieldwork');
  const saved = signal('Your workspace is ready.');
  return (
    <Card className="w-full">
      <CardHeader><CardTitle><Text as="h3" className="text-xl">Your workspace</Text></CardTitle><CardDescription>A name for what you’re making next.</CardDescription></CardHeader>
      <CardContent className="gap-3"><Label htmlFor="card-project-name">Project name</Label><Input id="card-project-name" value={name} className="w-full" onChange={(event) => { name.set(event.target.value); }} /><Text role="status" className="text-sm text-muted-foreground">{saved}</Text></CardContent>
      <CardFooter className="flex-wrap justify-between gap-4"><Text className="text-xs text-muted-foreground">Local state. Your source.</Text><Button disabled={name.get().trim().length === 0} onPress={() => { saved.set('Saved ' + name.get().trim()); }}>Save changes</Button></CardFooter>
    </Card>
  );
}
