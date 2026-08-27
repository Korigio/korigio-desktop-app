import { Link } from "react-router";
import { Page, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  message?: string | null;
};

/** Shared loading / not-found chrome for customer detail & edit routes. */
export function CustomerLoadState({ message }: Props) {
  const { t } = useI18n();

  if (!message) {
    return (
      <Page>
        <StatusMessage>{t("common.loading")}</StatusMessage>
      </Page>
    );
  }

  return (
    <Page>
      <StatusMessage tone="danger">{message}</StatusMessage>
      <Link className="text-sm text-primary hover:underline" to="/customers">
        {t("customers.backToList")}
      </Link>
    </Page>
  );
}
