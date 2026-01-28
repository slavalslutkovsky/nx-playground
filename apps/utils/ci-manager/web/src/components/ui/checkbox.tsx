import type { Component, ComponentProps } from 'solid-js';
import { splitProps } from 'solid-js';
import { cn } from '../../lib/utils';

export interface CheckboxProps extends Omit<ComponentProps<'input'>, 'type'> {
  label?: string;
}

const Checkbox: Component<CheckboxProps> = (props) => {
  const [local, others] = splitProps(props, ['class', 'label', 'id']);

  return (
    <div class="flex items-center space-x-2">
      <input
        type="checkbox"
        id={local.id}
        class={cn(
          'h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500 focus:ring-2',
          local.class,
        )}
        {...others}
      />
      {local.label && (
        <label
          for={local.id}
          class="text-sm font-medium text-gray-700 cursor-pointer select-none"
        >
          {local.label}
        </label>
      )}
    </div>
  );
};

export { Checkbox };
