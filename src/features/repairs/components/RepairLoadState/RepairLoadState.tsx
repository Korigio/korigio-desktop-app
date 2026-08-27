import { Link } from "react-router";
import { Page, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  message?: string | null;
};

export function RepairLoadState({ message }: Props) {
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
      <Link className="text-sm text-primary hover:underline" to="/repairs">
        {t("repairs.backToList")}
      </Link>
    </Page>
  );
}
