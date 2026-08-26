import type { InputHTMLAttributes } from "react";
import { cn } from "@/shared/utils/cn";

export type SelectFieldProps = InputHTMLAttributes<HTMLSelectElement>;

export function SelectField({ className, children, ...props }: SelectFieldProps) {
  return (
    <select
      className={cn(
        "w-full rounded-md border border-border bg-surface px-3 py-2 text-sm text-foreground",
        "focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary",
        "disabled:cursor-not-allowed disabled:opacity-50",
        className,
      )}
      {...props}
    >
      {children}
    </select>
  );
}
