import type { ReactNode } from "react";
import { cn } from "@/shared/utils/cn";

export type CardProps = {
  title?: string;
  actions?: ReactNode;
  children: ReactNode;
  className?: string;
};

export function Card({ title, actions, children, className }: CardProps) {
  return (
    <section
      className={cn(
        "rounded-md border border-border bg-surface p-4 sm:p-5",
        className,
      )}
    >
      {title || actions ? (
        <div className="mb-4 flex flex-wrap items-center justify-between gap-2">
          {title ? (
            <h2 className="text-base font-semibold text-foreground">{title}</h2>
          ) : (
            <span />
          )}
          {actions ? (
            <div className="flex flex-wrap items-center gap-2">{actions}</div>
          ) : null}
        </div>
      ) : null}
      {children}
    </section>
  );
}
