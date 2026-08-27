import type { RepairIntakeState } from "@/features/repairs/hooks/useRepairIntake";
import { Button, LinkButton, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  intake: RepairIntakeState;
};

export function IntakeDoneStep({ intake }: Props) {
  const { t } = useI18n();
  const repair = intake.createdRepair;

  if (!repair) {
    return null;
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-col gap-1">
        <h2 className="text-base font-medium">
          {t("repairs.intake.done.heading")}
        </h2>
        <StatusMessage tone="success">
          {t("repairs.intake.done.repairCreated").replace(
            "{number}",
            repair.repairNumber,
          )}
        </StatusMessage>
      </div>

      <div className="flex flex-wrap gap-2">
        <LinkButton to={`/repairs/${repair.id}/print`}>
          {t("repairs.intake.done.printEntrance")}
        </LinkButton>
        <LinkButton to={`/repairs/${repair.id}`} variant="secondary">
          {t("repairs.intake.done.openRepair")}
        </LinkButton>
        <Button type="button" variant="secondary" onClick={intake.startAnother}>
          {t("repairs.intake.done.startAnother")}
        </Button>
      </div>
    </div>
  );
}
