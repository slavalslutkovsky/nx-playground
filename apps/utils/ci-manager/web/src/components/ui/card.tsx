import type { Component, ComponentProps } from 'solid-js';
import { splitProps } from 'solid-js';
import { cn } from '../../lib/utils';

const Card: Component<ComponentProps<'div'>> = (props) => {
  const [local, others] = splitProps(props, ['class']);
  return (
    <div
      class={cn(
        'rounded-xl border border-gray-200 bg-white text-gray-900 shadow',
        local.class,
      )}
      {...others}
    />
  );
};

const CardHeader: Component<ComponentProps<'div'>> = (props) => {
  const [local, others] = splitProps(props, ['class']);
  return (
    <div class={cn('flex flex-col space-y-1.5 p-6', local.class)} {...others} />
  );
};

const CardTitle: Component<ComponentProps<'h3'>> = (props) => {
  const [local, others] = splitProps(props, ['class']);
  return (
    <h3
      class={cn(
        'text-xl font-semibold leading-none tracking-tight',
        local.class,
      )}
      {...others}
    />
  );
};

const CardDescription: Component<ComponentProps<'p'>> = (props) => {
  const [local, others] = splitProps(props, ['class']);
  return <p class={cn('text-sm text-gray-500', local.class)} {...others} />;
};

const CardContent: Component<ComponentProps<'div'>> = (props) => {
  const [local, others] = splitProps(props, ['class']);
  return <div class={cn('p-6 pt-0', local.class)} {...others} />;
};

const CardFooter: Component<ComponentProps<'div'>> = (props) => {
  const [local, others] = splitProps(props, ['class']);
  return (
    <div class={cn('flex items-center p-6 pt-0', local.class)} {...others} />
  );
};

export {
  Card,
  CardHeader,
  CardFooter,
  CardTitle,
  CardDescription,
  CardContent,
};
