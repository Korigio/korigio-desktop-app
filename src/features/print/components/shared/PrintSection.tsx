import type { ReactNode } from "react";

type Props = {
  title: string;
  children: ReactNode;
  className?: string;
};

export function PrintSection({ title, children, className = "mt-4" }: Props) {
  return (
    <section className={className}>
      <h2 className="text-sm font-semibold uppercase tracking-wide">{title}</h2>
      {children}
    </section>
  );
}
