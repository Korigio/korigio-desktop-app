import type { ReactNode } from "react";
import { cn } from "@/shared/utils/cn";

export type DefinitionItem = {
  label: string;
  value: ReactNode;
};

export type DefinitionListProps = {
  items: DefinitionItem[];
  className?: string;
};

export function DefinitionList({ items, className }: DefinitionListProps) {
  return (
    <dl className={cn("grid max-w-xl gap-4 text-sm", className)}>
      {items.map((item) => (
        <div key={item.label}>
          <dt className="font-medium text-muted">{item.label}</dt>
          <dd className="whitespace-pre-wrap">{item.value}</dd>
        </div>
      ))}
    </dl>
  );
}
