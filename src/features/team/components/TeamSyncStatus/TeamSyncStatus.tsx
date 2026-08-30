import type { SyncStatus } from "@/features/team/types/team";
import { syncErrorDisplay } from "@/features/team/utils/syncStatusMessage";
import { useI18n } from "@/shared/hooks/useI18n";
import { StatusMessage } from "@/ui";

type Props = {
  status: SyncStatus | null;
};

export function TeamSyncStatus({ status }: Props) {
  const { t } = useI18n();

  if (!status) {
    return null;
  }

  if (status.errorMessage) {
    return (
      <StatusMessage tone="danger">
        {syncErrorDisplay(status.errorMessage, t("team.sync.unreachable"))}
      </StatusMessage>
    );
  }

  if (status.peersOnline === 0) {
    return <StatusMessage>{t("team.sync.waiting")}</StatusMessage>;
  }

  return (
    <StatusMessage>
      {t("team.sync.connected").replace("{count}", String(status.peersOnline))}
    </StatusMessage>
  );
}
