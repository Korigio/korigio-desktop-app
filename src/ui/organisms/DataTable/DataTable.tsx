import type { ReactNode } from "react";
import { cn } from "@/shared/utils/cn";

export type DataTableProps = {
  header: ReactNode;
  children: ReactNode;
  className?: string;
};

export function DataTable({ header, children, className }: DataTableProps) {
  return (
    <div
      className={cn(
        "overflow-x-auto rounded-md border border-border",
        className,
      )}
    >
      <table className="min-w-full border-collapse text-left text-sm">
        <thead className="bg-background text-muted">{header}</thead>
        <tbody className="bg-surface">{children}</tbody>
      </table>
    </div>
  );
}
