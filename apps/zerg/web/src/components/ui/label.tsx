import type { Component, ComponentProps } from 'solid-js';
import { splitProps } from 'solid-js';
import { cn } from '../../lib/utils';

export interface LabelProps extends ComponentProps<'label'> {}

const Label: Component<LabelProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);
  return (
    // biome-ignore lint/a11y/noLabelWithoutControl: Label component accepts 'for' attribute via props spreading
    <label
      class={cn(
        'text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70',
        local.class,
      )}
      {...others}
    />
  );
};

export { Label };
