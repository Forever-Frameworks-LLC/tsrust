import { document, withTheme, View, Button, Text } from 'std:ui';
import { strictEqual } from 'std:testing';
import config from './tsrust.config.json';
import { Page } from './src/app/page';
import { Kbd, KbdGroup } from './src/components/ui/kbd';
import { Separator } from './src/components/ui/separator';

function main() {
  const html = document(withTheme(Page(), config.ui.theme), 'Content');
  strictEqual(html.includes('data-slot="kbd-group"'), true);
  strictEqual(html.includes('data-orientation="horizontal" data-horizontal'), true);
  strictEqual(html.includes('data-orientation="vertical" data-vertical'), true);
  strictEqual(html.includes('display:inline-flex'), true);
  strictEqual(html.includes('width:fit-content'), true);
  strictEqual(html.includes('font-size:12px'), true);
  strictEqual(html.includes('>Ctrl</span>'), true);
  const nested = document(<Button><KbdGroup><Kbd>A</Kbd>+<Kbd>B</Kbd></KbdGroup></Button>, 'Nested');
  strictEqual(nested.includes('<kbd'), true);
  const code = document(<View as="code"><Text>Composed</Text><Kbd>X</Kbd></View>, 'Code');
  strictEqual(code.includes('<code'), true);
  const themed = document(withTheme(<Kbd className="h-8 px-4 rounded-lg bg-primary text-primary-foreground text-base">Custom</Kbd>, config.ui.theme, 'dark'), 'Custom');
  strictEqual(themed.includes('height:32px'), true);
  console.log('Semantic content passed');
}
