import { LinkButton } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

export function NewCompanyButton() {
  const { t } = useI18n();
  return (
    <LinkButton to="/companies/new">{t("companies.actions.new")}</LinkButton>
  );
}
