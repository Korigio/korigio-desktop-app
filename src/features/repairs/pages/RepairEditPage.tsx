import { RepairForm } from "@/features/repairs/components/RepairForm";
import { RepairLoadState } from "@/features/repairs/components/RepairLoadState";
import { useRepairDetail } from "@/features/repairs/hooks/useRepairDetail";
import { useRepairForm } from "@/features/repairs/hooks/useRepairForm";
import type { Repair } from "@/features/repairs/types/repair";
import { Page, PageHeader, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = { repairId: string };

export function RepairEditPage({ repairId }: Props) {
  const { t } = useI18n();
  const { repair, loading, error } = useRepairDetail(repairId);

  if (loading) {
    return <RepairLoadState />;
  }

  if (error || !repair) {
    return <RepairLoadState message={error ?? t("repairs.notFound")} />;
  }

  return <RepairEditForm repair={repair} />;
}

function RepairEditForm({ repair }: { repair: Repair }) {
  const { t } = useI18n();
  const form = useRepairForm({ mode: "edit", repair });
  const isArchived = Boolean(repair.archivedAt);

  return (
    <Page>
      <PageHeader title={t("repairs.editTitle")} />
      <RepairForm
        form={form}
        submitLabel={t("repairs.actions.save")}
        disabled={isArchived}
        lockCustomer
        lockDevice
        hideStatus
      />
      {isArchived ? (
        <StatusMessage>{t("repairs.archivedEditHint")}</StatusMessage>
      ) : null}
    </Page>
  );
}
