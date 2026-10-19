import { Button as PrimitiveButton, ButtonProps as PrimitiveButtonProps, Node } from 'std:ui';

export interface ButtonProps extends PrimitiveButtonProps {
  variant?: 'default' | 'secondary' | 'outline' | 'destructive';
  size?: 'sm' | 'default' | 'lg';
}
/** Editable project button. Caller classes override defaults by property and state. */
export function Button(props: ButtonProps): Node {
  const variant = props.variant ?? 'default';
  const size = props.size ?? 'default';
  let classes = 'rounded text-center focus:ring ';
  if (variant === 'secondary') {
    classes += 'bg-secondary text-secondary-foreground hover:bg-accent active:bg-muted ';
  } else if (variant === 'outline') {
    classes += 'border border-border bg-background text-foreground hover:bg-accent active:bg-muted ';
  } else if (variant === 'destructive') {
    classes += 'bg-destructive text-destructive-foreground hover:bg-secondary active:bg-muted ';
  } else {
    classes += 'bg-primary text-primary-foreground hover:bg-secondary active:bg-accent ';
  }

  if (size === 'sm') {
    classes += 'px-2 py-2 ';
  } else if (size === 'lg') {
    classes += 'px-6 py-4 ';
  } else {
    classes += 'px-4 py-2 ';
  }

  classes += props.className ?? '';
  return PrimitiveButton({
    children: props.children, className: classes, disabled: props.disabled, onPress: props.onPress
  });
}
