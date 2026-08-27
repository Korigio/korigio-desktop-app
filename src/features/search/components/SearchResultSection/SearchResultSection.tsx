import type { ReactNode } from "react";
import { StatusMessage } from "@/ui";

export type SearchResultSectionProps = {
  title: string;
  emptyMessage: string;
  isEmpty: boolean;
  children: ReactNode;
};

export function SearchResultSection({
  title,
  emptyMessage,
  isEmpty,
  children,
}: SearchResultSectionProps) {
  return (
    <section className="flex flex-col gap-2">
      <h2 className="text-lg font-semibold">{title}</h2>
      {isEmpty ? (
        <StatusMessage>{emptyMessage}</StatusMessage>
      ) : (
        <ul className="divide-y divide-border rounded-md border border-border bg-surface">
          {children}
        </ul>
      )}
    </section>
  );
}
