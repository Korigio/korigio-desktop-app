import type { ReactNode } from "react";

type Props = {
  children: ReactNode;
};

export function PrintSheet({ children }: Props) {
  return (
    <article className="print-sheet mx-auto max-w-[210mm] bg-white text-black">
      {children}
    </article>
  );
}
