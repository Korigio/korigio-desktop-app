import { RepairDiagnosisSection } from "@/features/diagnosis/components/RepairDiagnosisSection";
import { RepairDetailActions } from "@/features/repairs/components/RepairDetailActions";
import { RepairDetailFields } from "@/features/repairs/components/RepairDetailFields";
import { RepairLoadState } from "@/features/repairs/components/RepairLoadState";
import { useRepairDetail } from "@/features/repairs/hooks/useRepairDetail";
import { Page, PageHeader } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";
import { useNavigate } from "react-router";

type Props = { repairId: number };

export function RepairDetailPage({ repairId }: Props) {
  const { t } = useI18n();
  const navigate = useNavigate();
  const { repair, loading, error, setRepair } = useRepairDetail(repairId);

  if (loading) {
    return <RepairLoadState />;
  }

  if (error || !repair) {
    return <RepairLoadState message={error ?? t("repairs.notFound")} />;
  }

  return (
    <Page>
      <PageHeader
        title={repair.repairNumber}
        description={t(`repairs.status.${repair.status}`)}
        actions={
          <RepairDetailActions
            repair={repair}
            onRepairChange={setRepair}
            onBack={() => navigate("/repairs")}
          />
        }
      />
      <RepairDetailFields repair={repair} />
      <RepairDiagnosisSection repairId={repair.id} />
    </Page>
  );
}
