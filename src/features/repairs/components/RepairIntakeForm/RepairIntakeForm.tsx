import { IntakeCustomerStep } from "@/features/repairs/components/IntakeCustomerStep";
import { IntakeDeviceStep } from "@/features/repairs/components/IntakeDeviceStep";
import { IntakeDetailsStep } from "@/features/repairs/components/IntakeDetailsStep";
import { IntakeReviewStep } from "@/features/repairs/components/IntakeReviewStep";
import { IntakeDoneStep } from "@/features/repairs/components/IntakeDoneStep";
import {
  INTAKE_STEPS,
  type RepairIntakeState,
} from "@/features/repairs/hooks/useRepairIntake";
import { StatusMessage, Stepper, WizardNav } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  intake: RepairIntakeState;
};

export function RepairIntakeForm({ intake }: Props) {
  const { t } = useI18n();

  const steps = INTAKE_STEPS.map((id) => ({
    id,
    label: t(`repairs.intake.steps.${id}`),
  }));

  const showBack = intake.step !== "customer" && intake.step !== "done";
  const showNav = intake.step !== "done";

  const primaryLabel =
    intake.step === "review"
      ? t("repairs.intake.nav.create")
      : t("repairs.intake.nav.continue");

  return (
    <div className="flex max-w-xl flex-col gap-6">
      <Stepper steps={steps} currentStepId={intake.step} />

      {intake.error ? (
        <StatusMessage tone="danger">{intake.error}</StatusMessage>
      ) : null}

      {intake.step === "customer" ? (
        <IntakeCustomerStep intake={intake} />
      ) : null}
      {intake.step === "device" ? <IntakeDeviceStep intake={intake} /> : null}
      {intake.step === "details" ? (
        <IntakeDetailsStep intake={intake} />
      ) : null}
      {intake.step === "review" ? <IntakeReviewStep intake={intake} /> : null}
      {intake.step === "done" ? <IntakeDoneStep intake={intake} /> : null}

      {showNav ? (
        <WizardNav
          showBack={showBack}
          backLabel={t("repairs.intake.nav.back")}
          onBack={intake.goBack}
          primaryLabel={primaryLabel}
          onPrimary={() => void intake.continuePrimary()}
          primaryDisabled={intake.submitting}
          primaryRef={
            intake.step === "review" ? intake.createButtonRef : undefined
          }
          hint={t("repairs.intake.shortcutHint")}
        />
      ) : null}
    </div>
  );
}
