import type { ReactNode } from "react";

type Props = {
  runningCompanyName: string;
  runningTitle: string;
  runningRepairNumber: string;
  footerLine: string;
  children: ReactNode;
};

export function PrintDocumentFrame({
  runningCompanyName,
  runningTitle,
  runningRepairNumber,
  footerLine,
  children,
}: Props) {
  return (
    <table className="print-document">
      <thead>
        <tr>
          <td>
            <div className="flex h-[8mm] items-center justify-between gap-4 overflow-hidden border-b border-[#cccccc] text-xs leading-none text-neutral-700">
              <span className="min-w-0 truncate">{runningCompanyName}</span>
              <span className="shrink-0 truncate">
                {runningTitle} · {runningRepairNumber}
              </span>
            </div>
          </td>
        </tr>
      </thead>
      <tbody>
        <tr>
          <td>{children}</td>
        </tr>
      </tbody>
      <tfoot>
        <tr>
          <td>
            <div className="border-t border-[#cccccc] pt-1 text-[10px] leading-tight text-neutral-600">
              {footerLine}
            </div>
          </td>
        </tr>
      </tfoot>
    </table>
  );
}
