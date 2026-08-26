import type { ReactNode } from "react";
import { cn } from "@/shared/utils/cn";

export type StatusMessageProps = {
  tone?: "muted" | "danger";
  children: ReactNode;
  className?: string;
};

export function StatusMessage({
  tone = "muted",
  children,
  className,
}: StatusMessageProps) {
  return (
    <p
      className={cn(
        "text-sm",
        tone === "muted" && "text-muted",
        tone === "danger" && "text-red-700",
        className,
      )}
    >
      {children}
    </p>
  );
}
