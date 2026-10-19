import { Text, Node, signal } from 'std:ui';
import { Card, CardHeader, CardTitle, CardDescription, CardContent, CardFooter } from '../components/ui/card';
import { Button } from '../components/ui/button';
import { Input } from '../components/ui/input';
import { Label } from '../components/ui/label';

export function CardProject(): Node {
  const name = signal('My project');
  const saved = signal('Project name: My project');
  return (
    <Card className="w-full">
      <CardHeader><CardTitle><Text as="h3">Project settings</Text></CardTitle><CardDescription>Update the name shown in this preview.</CardDescription></CardHeader>
      <CardContent className="gap-2"><Label htmlFor="card-project-name">Project name</Label><Input id="card-project-name" value={name} onChange={(event) => { name.set(event.target.value); }} /><Text role="status">{saved}</Text></CardContent>
      <CardFooter><Button disabled={name.get().length === 0} onPress={() => { saved.set('Project name: ' + name.get()); }}>Apply name</Button></CardFooter>
    </Card>
  );
}
