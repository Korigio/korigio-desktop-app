import { PrintPageToolbar } from "@/features/print/components/PrintPageToolbar";
import { SummaryPrintDocument } from "@/features/print/components/SummaryPrintDocument";
import { usePrintPageActions } from "@/features/print/hooks/usePrintPageActions";
import { useSummaryPrintReport } from "@/features/print/hooks/useSummaryPrintReport";
import { StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = { repairId: string };

export function SummaryPrintPage({ repairId }: Props) {
  const { t } = useI18n();
  const { report, loading, error } = useSummaryPrintReport(repairId);
  const actions = usePrintPageActions({
    repairId,
    closeLabelKey: "print.actions.close",
  });

  return (
    <div className="min-h-full bg-neutral-100 text-neutral-900">
      <PrintPageToolbar
        onPrint={actions.print}
        onClose={actions.close}
        busy={actions.busy}
        error={actions.error}
        closeLabel={actions.closeLabel}
      />

      <div className="p-4 print:p-0">
        {loading ? <StatusMessage>{t("common.loading")}</StatusMessage> : null}
        {error ? <StatusMessage tone="danger">{error}</StatusMessage> : null}
        {report ? <SummaryPrintDocument report={report} /> : null}
      </div>
    </div>
  );
}
