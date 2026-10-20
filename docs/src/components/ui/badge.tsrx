// License: MIT. See ../../../attributions/shadcn-MIT.txt.
import { View, ViewProps, Node, ReactiveValue, useProp } from 'std:ui';

export type BadgeVariant = 'default' | 'secondary' | 'destructive' | 'outline' | 'ghost' | 'link';

export interface BadgeVariantOptions {
  /** Omission selects default. null omits variant styling. */
  variant?: BadgeVariant | null;
  /** Additional classes override matching default properties. */
  className?: string;
}
/** Returns Nova badge classes without creating a node. Defaults to the primary palette. No arguments selects default; null variant keeps base sizing only. */
export function badgeVariants(options: BadgeVariantOptions = {}): string {
  const variant = options.variant === undefined ? 'default' : options.variant;
  let palette = '';
  if (variant === 'default') {
    palette = 'bg-primary text-primary-foreground [a]:hover:bg-primary/80 ';
  }

  if (variant === 'secondary') {
    palette = 'bg-secondary text-secondary-foreground [a]:hover:bg-secondary/80 ';
  }

  if (variant === 'destructive') {
    palette = 'bg-destructive/10 text-destructive [a]:hover:bg-destructive/20 focus-visible:ring-destructive/20 dark:focus-visible:ring-destructive/40 dark:bg-destructive/20 ';
  }

  if (variant === 'outline') {
    palette = 'border-border text-foreground [a]:hover:bg-muted [a]:hover:text-muted-foreground ';
  }

  if (variant === 'ghost') {
    palette = 'hover:bg-muted hover:text-muted-foreground dark:hover:bg-muted/50 ';
  }

  if (variant === 'link') {
    palette = 'text-primary underline-offset-4 hover:underline ';
  }

  return 'group/badge inline-flex flex-row w-fit shrink-0 items-center justify-center overflow-hidden whitespace-nowrap h-5 gap-1 rounded-4xl border border-transparent px-2 py-0.5 text-xs font-medium focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 ' + palette + (options.className ?? '');
}

export interface BadgeProps extends ViewProps<ReactiveValue<string>, ReactiveValue<string>> {
  /** Live six-way palette. Undefined selects default; null removes variant styling and data-variant. */
  variant?: ReactiveValue<BadgeVariant | null>;
}
/** Compact status label. Defaults to span without an implicit role or action. render replaces the node once; forward styled props onto a Link or Button for an interactive badge. */
export function Badge(props: BadgeProps): Node {
  const variant = useProp<BadgeVariant | null>(props.variant === undefined ? 'default' : props.variant);
  const classes = useProp<string>(props.className ?? '');
  const tag = clone(props.as) ?? 'span';
  const slot = clone(props['data-slot']) ?? 'badge';
  props.variant = undefined;
  props.className = undefined;
  return (
    <View {...props} as={tag} data-slot={slot} data-variant={variant.get() ?? ''}
      className={badgeVariants({ variant: variant.get(), className: classes.get() })} />
  );
}
