// License: MIT. See ../../../attributions/shadcn-MIT.txt.
import { View, Text, ViewProps, TextProps, Node, ReactiveValue, useProp } from 'std:ui';

export interface EmptyProps extends ViewProps<ReactiveValue<string>, ReactiveValue<string>> {}

export interface EmptyHeaderProps extends EmptyProps {}

export interface EmptyContentProps extends EmptyProps {}

export interface EmptyTitleProps extends TextProps<ReactiveValue<string>, ReactiveValue<string>> {}

export interface EmptyDescriptionProps extends TextProps<ReactiveValue<string>, ReactiveValue<string>> {}

export type EmptyMediaVariant = 'default' | 'icon';

export interface EmptyMediaProps extends EmptyProps {
  /** Live media styling. Undefined selects default; null omits variant styling. */
  variant?: ReactiveValue<EmptyMediaVariant | null>;
}

/** Centered empty-state container. Border style defaults to dashed; add border for a visible edge. */
export function Empty(props: EmptyProps): Node {
  const classes = useProp<string>(props.className ?? '');
  const slot = clone(props['data-slot']) ?? 'empty';
  props.className = undefined;
  return (<View {...props} data-slot={slot} className={'flex w-full min-w-0 flex-1 flex-col items-center justify-center text-center whitespace-normal text-balance gap-4 rounded-xl border-dashed p-6 ' + classes.get()}/>);
}

/** Groups an empty state's media, title and description within the small content width. */
export function EmptyHeader(props: EmptyHeaderProps): Node {
  const classes = useProp<string>(props.className ?? '');
  const slot = clone(props['data-slot']) ?? 'empty-header';
  props.className = undefined;
  return (<View {...props} data-slot={slot} className={'flex max-w-sm flex-col items-center gap-2 ' + classes.get()}/>);
}

/** Media slot. icon uses a muted square and supplies small sizing only when an SVG has no size class. */
export function EmptyMedia(props: EmptyMediaProps): Node {
  const variant = useProp<EmptyMediaVariant | null>(props.variant === undefined ? 'default' : props.variant);
  const classes = useProp<string>(props.className ?? '');
  const slot = clone(props['data-slot']) ?? 'empty-icon';
  props.className = undefined;
  props.variant = undefined;
  return (<View {...props} data-slot={slot} data-variant={variant.get() ?? ''} className={'flex flex-row shrink-0 items-center justify-center mb-2 [&_svg]:pointer-events-none [&_svg]:shrink-0 ' + (variant.get() === 'icon' ? 'bg-muted text-foreground size-8 rounded-lg [&_svg:not([class*=\'size-\'])]:size-4 ' : variant.get() === 'default' ? 'bg-transparent ' : '') + classes.get()}/>);
}

/** Empty-state title. Defaults to a div; choose a heading tag when the document hierarchy calls for one. */
export function EmptyTitle(props: EmptyTitleProps): Node {
  const classes = useProp<string>(props.className ?? '');
  const slot = clone(props['data-slot']) ?? 'empty-title';
  const tag = clone(props.as) ?? 'div';
  props.className = undefined;
  return (<Text {...props} as={tag} data-slot={slot} className={'font-heading text-sm font-medium tracking-tight ' + classes.get()}/>);
}

/** Flowing description text with underlined direct links. Defaults to div and preserves inline text selection. */
export function EmptyDescription(props: EmptyDescriptionProps): Node {
  const classes = useProp<string>(props.className ?? '');
  const slot = clone(props['data-slot']) ?? 'empty-description';
  const tag = clone(props.as) ?? 'div';
  props.className = undefined;
  return (<Text {...props} as={tag} data-slot={slot} className={'text-muted-foreground text-sm/relaxed [&>a]:underline [&>a]:underline-offset-4 [&>a:hover]:text-primary ' + classes.get()}/>);
}

/** Stacks actions or other empty-state content within the small content width. */
export function EmptyContent(props: EmptyContentProps): Node {
  const classes = useProp<string>(props.className ?? '');
  const slot = clone(props['data-slot']) ?? 'empty-content';
  props.className = undefined;
  return (<View {...props} data-slot={slot} className={'flex w-full max-w-sm min-w-0 flex-col items-center text-balance gap-2.5 text-sm ' + classes.get()}/>);
}
