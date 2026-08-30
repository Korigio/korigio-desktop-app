import { SummaryPrintDocument } from "@/features/print/components/SummaryPrintDocument";
import { useSummaryPrintReport } from "@/features/print/hooks/useSummaryPrintReport";
import { Button, StatusMessage } from "@/ui";
import { useNavigate } from "react-router";

type Props = { repairId: string };

export function SummaryPrintPage({ repairId }: Props) {
  const navigate = useNavigate();
  const { report, loading, error } = useSummaryPrintReport(repairId);

  return (
    <div className="min-h-full bg-neutral-100 text-foreground">
      <div className="print:hidden flex flex-wrap items-center gap-2 border-b border-border bg-surface px-4 py-3">
        <Button type="button" onClick={() => window.print()}>
          Print
        </Button>
        <Button
          type="button"
          variant="secondary"
          onClick={() => navigate(`/repairs/${repairId}`)}
        >
          Back to repair
        </Button>
      </div>

      <div className="p-4 print:p-0">
        {loading ? <StatusMessage>Loading…</StatusMessage> : null}
        {error ? <StatusMessage tone="danger">{error}</StatusMessage> : null}
        {report ? <SummaryPrintDocument report={report} /> : null}
      </div>
    </div>
  );
}
