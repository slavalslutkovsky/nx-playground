import type { Component } from "solid-js";
import { Badge } from "~/components/ui/badge";
import type { CloudProvider } from "~/types";

interface ProviderBadgeProps {
  provider: CloudProvider;
}

const ProviderBadge: Component<ProviderBadgeProps> = (props) => {
  return (
    <Badge variant={props.provider}>
      {props.provider.toUpperCase()}
    </Badge>
  );
};

export { ProviderBadge };
