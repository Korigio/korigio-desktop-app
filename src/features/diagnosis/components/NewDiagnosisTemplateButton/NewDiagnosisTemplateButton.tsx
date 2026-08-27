import { LinkButton } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

export function NewDiagnosisTemplateButton() {
  const { t } = useI18n();
  return (
    <LinkButton to="/diagnosis-templates/new">
      {t("diagnosis.actions.new")}
    </LinkButton>
  );
}
