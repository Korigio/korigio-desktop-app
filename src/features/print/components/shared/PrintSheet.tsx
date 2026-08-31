import type { CSSProperties, ReactNode } from "react";

type Props = {
  children: ReactNode;
  pageLabel?: string;
};

export function PrintSheet({ children, pageLabel }: Props) {
  const style = pageLabel
    ? ({ "--print-page-label": pageLabel } as CSSProperties)
    : undefined;

  return (
    <article
      className="print-sheet mx-auto w-[210mm] min-h-[297mm] bg-white px-[16mm] py-[16mm] text-neutral-900 shadow-sm print:w-full print:max-w-none print:min-h-0 print:p-0 print:shadow-none"
      style={style}
    >
      {children}
    </article>
  );
}
