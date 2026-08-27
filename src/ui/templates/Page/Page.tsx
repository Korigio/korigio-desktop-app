import type { ReactNode } from "react";
import { cn } from "@/shared/utils/cn";

export type PageProps = {
  children: ReactNode;
  className?: string;
};

/** Standard feature page shell — padding + vertical rhythm. */
export function Page({ children, className }: PageProps) {
  return (
    <main className={cn("flex flex-col gap-6 p-6", className)}>{children}</main>
  );
}
