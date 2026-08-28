import type { Repair } from "@/features/repairs/types/repair";
import { Card, DefinitionList } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

const EMPTY = "—";

type Props = {
  repair: Repair;
};

/** Intake / entrance fields only — excludes diagnosis and workshop data. */
export function RepairEntranceInfoCard({ repair }: Props) {
  const { t } = useI18n();

  return (
    <Card title={t("repairs.detail.sections.entrance")}>
      <DefinitionList
        items={[
          {
            label: t("repairs.fields.receivedAt"),
            value: repair.receivedAt.slice(0, 10),
          },
          {
            label: t("repairs.fields.reportedProblem"),
            value: repair.reportedProblem?.trim() || EMPTY,
          },
          {
            label: t("repairs.fields.accessoriesReceived"),
            value: repair.accessoriesReceived?.trim() || EMPTY,
          },
          {
            label: t("repairs.fields.deviceCondition"),
            value: repair.deviceCondition?.trim() || EMPTY,
          },
          {
            label: t("repairs.fields.expectedPickupAt"),
            value: repair.expectedPickupAt ?? EMPTY,
          },
          {
            label: t("repairs.fields.notes"),
            value: repair.notes?.trim() || EMPTY,
          },
        ]}
      />
    </Card>
  );
}
