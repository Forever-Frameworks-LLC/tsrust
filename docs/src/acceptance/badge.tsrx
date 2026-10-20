import { View, Text, Button, Link, document } from 'std:ui';
import { strictEqual } from 'std:testing';
import { Badge, badgeVariants, BadgeVariant } from '../components/ui/badge';

function main() {
  const defaults = badgeVariants();
  strictEqual(defaults.includes('bg-primary'), true);
  strictEqual(badgeVariants({
    variant: null
  }).includes('bg-primary'), false);
  const variants: BadgeVariant[] = ['default', 'secondary', 'destructive', 'outline', 'ghost', 'link'];
  for (const variant of variants) {
    const html = document(<Badge variant={clone(variant)}>Status</Badge>, 'Badge');
    strictEqual(html.includes('data-variant="' + variant + '"'), true);
    strictEqual(html.includes('white-space:nowrap'), true);
    strictEqual(html.includes('<span'), true);
  }

  const neutral = document(<Badge variant={null}>Neutral</Badge>, 'Neutral');
  strictEqual(neutral.includes('data-variant='), false);
  const link = document(<Badge render={(props) => (<Link href="/docs" className={props.className} data-slot={props['data-slot']}>Docs</Link>)} />, 'Link');
  strictEqual(link.includes('<a '), true);
  strictEqual(link.includes('<span class="ui group/badge'), false, 'Replacement Link has no Badge span wrapper');
  const button = document(<Badge render={(props) => (<Button className={props.className} onPress={() => {}}>Action</Button>)} />, 'Button');
  strictEqual(button.includes('<button'), true);
  strictEqual(button.includes('<span class="ui group/badge'), false, 'Replacement Button has no Badge span wrapper');
  console.log('Badge composition passed');
}
