import { DiagnosisPrintDocument } from "@/features/print/components/DiagnosisPrintDocument";
import { useDiagnosisPrintReport } from "@/features/print/hooks/useDiagnosisPrintReport";
import { Button, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";
import { useNavigate } from "react-router";

type Props = { repairId: number };

export function DiagnosisPrintPage({ repairId }: Props) {
  const { t } = useI18n();
  const navigate = useNavigate();
  const { report, loading, error } = useDiagnosisPrintReport(repairId);

  return (
    <div className="min-h-full bg-neutral-100 text-foreground">
      <div className="print:hidden flex flex-wrap items-center gap-2 border-b border-border bg-surface px-4 py-3">
        <Button type="button" onClick={() => window.print()}>
          {t("print.actions.print")}
        </Button>
        <Button
          type="button"
          variant="secondary"
          onClick={() => navigate(`/repairs/${repairId}`)}
        >
          {t("print.actions.closeDiagnosis")}
        </Button>
      </div>

      <div className="p-4 print:p-0">
        {loading ? (
          <StatusMessage>{t("common.loading")}</StatusMessage>
        ) : null}
        {error ? <StatusMessage tone="danger">{error}</StatusMessage> : null}
        {report ? <DiagnosisPrintDocument report={report} /> : null}
      </div>
    </div>
  );
}
