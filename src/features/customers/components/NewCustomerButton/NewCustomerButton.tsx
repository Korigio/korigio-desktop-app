import { LinkButton } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

export function NewCustomerButton() {
  const { t } = useI18n();
  return (
    <LinkButton to="/customers/new">{t("customers.actions.new")}</LinkButton>
  );
}
