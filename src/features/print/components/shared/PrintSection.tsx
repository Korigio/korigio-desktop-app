import type { ReactNode } from "react";
import { cn } from "@/shared/utils/cn";

type Props = {
  title: string;
  children: ReactNode;
  className?: string;
};

export function PrintSection({ title, children, className }: Props) {
  return (
    <section className={cn("mt-6", className)}>
      <h2 className="border-b border-[#cccccc] pb-1 text-sm font-semibold uppercase tracking-wide">
        {title}
      </h2>
      {children}
    </section>
  );
}
