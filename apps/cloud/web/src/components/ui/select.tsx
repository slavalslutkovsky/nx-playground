import { splitProps, type JSX, type Component } from "solid-js";
import { cn } from "~/lib/utils";

export interface SelectProps extends JSX.SelectHTMLAttributes<HTMLSelectElement> {}

const Select: Component<SelectProps> = (props) => {
  const [local, others] = splitProps(props, ["class", "children"]);

  return (
    <select
      class={cn(
        "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50",
        local.class
      )}
      {...others}
    >
      {local.children}
    </select>
  );
};

export { Select };
