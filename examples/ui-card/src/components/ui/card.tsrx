/*
MIT License

Copyright (c) 2023 shadcn

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

Adapted from shadcn-ui/ui daaf156ee00ed91cc7e6bdd937c1b4973152d141, Base UI + Nova.
Portable content parts; image children and inline-size containment remain unsupported.
*/
import { View, ViewProps, Node } from 'std:ui';

export interface CardProps extends ViewProps {
  /** Selects default or small spacing. Defaults to default. */
  size?: 'default' | 'sm';
}

/** Nova card surface for owned portable content. Override className or edit this local source. Image edge variants and inline-size containment are not implemented. */
export function Card(props: CardProps): Node {
  const size = clone(props['data-size']) ?? String(props.size ?? 'default');
  const slot = clone(props['data-slot']) ?? 'card';
  const classes = 'group/card flex flex-col ring-foreground/10 bg-card text-card-foreground gap-(--card-spacing) overflow-hidden rounded-xl py-(--card-spacing) text-sm ring-1 [--card-spacing:--spacing(4)] has-data-[slot=card-footer]:pb-0 data-[size=sm]:[--card-spacing:--spacing(3)] ' + (clone(props.className) ?? '');
  return (<View {...props} data-slot={slot} data-size={size} className={classes} />);
}

/** Header grid detects action/description slots. Container queries and inline-size containment are unsupported. */
export function CardHeader(props: ViewProps): Node {
  const slot = clone(props['data-slot']) ?? 'card-header';
  const classes = 'group/card-header grid auto-rows-min items-start has-data-[slot=card-action]:grid-cols-[1fr_auto] has-data-[slot=card-description]:grid-rows-[auto_auto] gap-1 rounded-t-xl px-(--card-spacing) [.border-b]:pb-(--card-spacing) ' + (clone(props.className) ?? '');
  return (<View {...props} data-slot={slot} className={classes} />);
}

/** Visible title text. Use a semantic heading child when this section needs a heading role. Custom named heading-font assets are unsupported. */
export function CardTitle(props: ViewProps): Node {
  const slot = clone(props['data-slot']) ?? 'card-title';
  const classes = 'text-base leading-snug font-medium group-data-[size=sm]/card:text-sm ' + (clone(props.className) ?? '');
  return (<View {...props} data-slot={slot} className={classes} />);
}

/** Muted description content; does not automatically label a control. */
export function CardDescription(props: ViewProps): Node {
  const slot = clone(props['data-slot']) ?? 'card-description';
  const classes = 'text-muted-foreground text-sm ' + (clone(props.className) ?? '');
  return (<View {...props} data-slot={slot} className={classes} />);
}

/** Action slot spans both header rows without changing keyboard or source order. */
export function CardAction(props: ViewProps): Node {
  const slot = clone(props['data-slot']) ?? 'card-action';
  const classes = 'col-start-2 row-span-2 row-start-1 self-start justify-self-end ' + (clone(props.className) ?? '');
  return (<View {...props} data-slot={slot} className={classes} />);
}

/** Card body inherits the card's scoped spacing; children retain their ordinary semantics. */
export function CardContent(props: ViewProps): Node {
  const slot = clone(props['data-slot']) ?? 'card-content';
  const classes = 'px-(--card-spacing) ' + (clone(props.className) ?? '');
  return (<View {...props} data-slot={slot} className={classes} />);
}

/** Muted footer with a top border, lower corners and inherited padding. */
export function CardFooter(props: ViewProps): Node {
  const slot = clone(props['data-slot']) ?? 'card-footer';
  const classes = 'flex flex-row items-center bg-muted/50 rounded-b-xl border-t p-(--card-spacing) ' + (clone(props.className) ?? '');
  return (<View {...props} data-slot={slot} className={classes} />);
}
