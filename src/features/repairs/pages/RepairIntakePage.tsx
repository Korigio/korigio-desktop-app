import { useEffect } from "react";
import { RepairIntakeForm } from "@/features/repairs/components/RepairIntakeForm";
import { useRepairIntake } from "@/features/repairs/hooks/useRepairIntake";
import { Page, PageHeader } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

export function RepairIntakePage() {
  const { t } = useI18n();
  const intake = useRepairIntake();
  const { step, submitting, continuePrimary } = intake;

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (!(event.ctrlKey || event.metaKey) || event.key !== "Enter") {
        return;
      }
      if (step === "done" || submitting) {
        return;
      }
      event.preventDefault();
      void continuePrimary();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [continuePrimary, step, submitting]);

  return (
    <Page>
      <PageHeader
        title={t("repairs.intake.title")}
        description={t("repairs.intake.subtitle")}
      />
      <RepairIntakeForm intake={intake} />
    </Page>
  );
}
