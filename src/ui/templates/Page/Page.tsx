import type { ReactNode } from "react";
import { cn } from "@/shared/utils/cn";

export type PageProps = {
  children: ReactNode;
  className?: string;
};

/** Standard feature page shell — padding + vertical rhythm. */
export function Page({ children, className }: PageProps) {
  return (
    <main className={cn("flex min-w-0 flex-col gap-4 p-4 sm:gap-6 sm:p-6", className)}>
      {children}
    </main>
  );
}
