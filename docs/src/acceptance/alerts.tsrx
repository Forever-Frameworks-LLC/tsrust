import { View, Text, Button, document } from 'std:ui';
import { strictEqual, throws } from 'std:testing';
import { Alert, AlertTitle, AlertDescription, AlertAction } from '../components/ui/alert';

function main() {
  const html = document(<Alert variant="destructive" id="notice"><AlertTitle>Connection failed</AlertTitle><AlertDescription>Check the address.</AlertDescription><AlertAction><Button>Retry</Button></AlertAction></Alert>, 'Alert');
  for (const text of ['role="alert"', 'data-variant="destructive"', 'data-slot="alert-action"', 'position:absolute', 'top:8px', 'right:8px', 'padding-right:72px', 'Connection failed', 'Check the address.', 'Retry']) {
    strictEqual(html.includes(text), true);
  }

  const polite = document(<Alert role="status" className="p-4"><Text>Saved</Text></Alert>, 'Status');
  strictEqual(polite.includes('role="status"'), true);
  strictEqual(polite.includes('data-variant="default"'), true);
  throws(() => {
    const invalid = <View data-variant="bad value"/>;
  });
  throws(() => {
    const invalid = <Text><Text className="absolute top-2">Bad inline box</Text></Text>;
  });
  console.log('Alert composition passed');
}
