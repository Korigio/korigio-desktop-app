import { PrintPageToolbar } from "@/features/print/components/PrintPageToolbar";
import { RepairPrintDocument } from "@/features/print/components/RepairPrintDocument";
import { usePrintPageActions } from "@/features/print/hooks/usePrintPageActions";
import { useRepairPrintReport } from "@/features/print/hooks/useRepairPrintReport";
import { StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = { repairId: string };

export function RepairPrintPage({ repairId }: Props) {
  const { t } = useI18n();
  const { report, loading, error } = useRepairPrintReport(repairId);
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
        {report ? <RepairPrintDocument report={report} /> : null}
      </div>
    </div>
  );
}
