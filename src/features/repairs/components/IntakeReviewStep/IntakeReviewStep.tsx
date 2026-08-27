import type { RepairIntakeState } from "@/features/repairs/hooks/useRepairIntake";
import { DefinitionList } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  intake: RepairIntakeState;
};

export function IntakeReviewStep({ intake }: Props) {
  const { t } = useI18n();

  return (
    <div className="flex flex-col gap-3">
      <h2 className="text-base font-medium">
        {t("repairs.intake.review.heading")}
      </h2>
      <DefinitionList items={intake.reviewItems} />
    </div>
  );
}
