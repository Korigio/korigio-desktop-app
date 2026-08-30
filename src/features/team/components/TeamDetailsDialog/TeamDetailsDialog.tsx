import type { ReactNode } from "react";
import { TeamMemberTable } from "@/features/team/components/TeamMemberTable";
import { TeamSyncStatus } from "@/features/team/components/TeamSyncStatus";
import type { SyncStatus, TeamMember } from "@/features/team/types/team";
import { useI18n } from "@/shared/hooks/useI18n";
import { Dialog, StatusMessage } from "@/ui";

type Props = {
  open: boolean;
  teamName: string;
  pin: string | null;
  members: TeamMember[];
  syncStatus: SyncStatus | null;
  onOpenChange: (open: boolean) => void;
  footer?: ReactNode;
};

export function TeamDetailsDialog({
  open,
  teamName,
  pin,
  members,
  syncStatus,
  onOpenChange,
  footer,
}: Props) {
  const { t } = useI18n();

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      title={teamName}
      closeLabel={t("common.close")}
      className="w-[min(100%-2rem,40rem)]"
    >
      <div className="flex flex-col gap-4">
        {pin ? (
          <div className="flex flex-col gap-1">
            <p className="text-xs font-medium text-muted">{t("team.pin.label")}</p>
            <p className="select-all text-2xl font-semibold tracking-widest">
              {pin}
            </p>
            <p className="text-xs text-muted">{t("team.pin.help")}</p>
          </div>
        ) : (
          <StatusMessage>{t("team.pin.unavailable")}</StatusMessage>
        )}
        <TeamSyncStatus status={syncStatus} />
        <div className="flex flex-col gap-2">
          <p className="text-xs font-medium text-muted">{t("team.members.title")}</p>
          <TeamMemberTable members={members} />
          <StatusMessage>{t("team.members.gigsHint")}</StatusMessage>
        </div>
        {footer}
      </div>
    </Dialog>
  );
}
