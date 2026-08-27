import type { Repair } from "@/features/repairs/types/repair";
import { DefinitionList, LinkButton } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

const EMPTY = "—";

type Props = {
  repair: Repair;
};

export function RepairDetailFields({ repair }: Props) {
  const { t } = useI18n();

  return (
    <DefinitionList
      items={[
        {
          label: t("repairs.fields.repairNumber"),
          value: repair.repairNumber,
        },
        {
          label: t("repairs.fields.status"),
          value: t(`repairs.status.${repair.status}`),
        },
        {
          label: t("repairs.fields.customer"),
          value: (
            <LinkButton
              to={`/customers/${repair.customerId}`}
              variant="secondary"
              className="px-2 py-1 text-xs"
            >
              #{repair.customerId}
            </LinkButton>
          ),
        },
        {
          label: t("repairs.fields.device"),
          value: (
            <LinkButton
              to={`/devices/${repair.deviceId}`}
              variant="secondary"
              className="px-2 py-1 text-xs"
            >
              #{repair.deviceId}
            </LinkButton>
          ),
        },
        {
          label: t("repairs.fields.receivedAt"),
          value: repair.receivedAt,
        },
        {
          label: t("repairs.fields.reportedProblem"),
          value: repair.reportedProblem ?? EMPTY,
        },
        {
          label: t("repairs.fields.accessoriesReceived"),
          value: repair.accessoriesReceived ?? EMPTY,
        },
        {
          label: t("repairs.fields.deviceCondition"),
          value: repair.deviceCondition ?? EMPTY,
        },
        {
          label: t("repairs.fields.expectedPickupAt"),
          value: repair.expectedPickupAt ?? EMPTY,
        },
        {
          label: t("repairs.fields.diagnosisNotes"),
          value: repair.diagnosisNotes ?? EMPTY,
        },
        {
          label: t("repairs.fields.workPerformed"),
          value: repair.workPerformed ?? EMPTY,
        },
        {
          label: t("repairs.fields.notes"),
          value: repair.notes ?? EMPTY,
        },
        {
          label: t("repairs.fields.readyAt"),
          value: repair.readyAt ?? EMPTY,
        },
        {
          label: t("repairs.fields.collectedAt"),
          value: repair.collectedAt ?? EMPTY,
        },
      ]}
    />
  );
}
