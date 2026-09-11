import type { RepairIntakeState } from "@/features/repairs/hooks/useRepairIntake";
import { Card, DefinitionList } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  intake: RepairIntakeState;
};

export function IntakeReviewStep({ intake }: Props) {
  const { t } = useI18n();

  return (
    <Card title={t("repairs.intake.review.heading")}>
      <DefinitionList items={intake.reviewItems} />
    </Card>
  );
}
