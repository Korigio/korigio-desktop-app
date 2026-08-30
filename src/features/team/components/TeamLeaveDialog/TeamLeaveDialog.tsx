import { useSession } from "@/features/staff/hooks/useSession";
import { teamApi } from "@/features/team/api/teamApi";
import { useI18n } from "@/shared/hooks/useI18n";
import { useModalAsyncAction } from "@/shared/hooks/useModalAsyncAction";
import { Button, Dialog, StatusMessage } from "@/ui";

type Props = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onLeft: () => Promise<void>;
};

export function TeamLeaveDialog({ open, onOpenChange, onLeft }: Props) {
  const { t } = useI18n();
  const { refresh } = useSession();
  const leaveAction = useModalAsyncAction(open);

  async function confirmLeave() {
    const done = await leaveAction.run(async () => {
      await teamApi.leave();
      await refresh();
      await onLeft();
    }, t("team.errors.lastAdminLeave"));
    if (done !== null) {
      onOpenChange(false);
    }
  }

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      title={t("team.leave.title")}
      description={t("team.leave.confirm")}
      closeLabel={t("common.close")}
    >
      <div className="flex flex-col gap-3">
        {leaveAction.error ? (
          <StatusMessage tone="danger">{leaveAction.error}</StatusMessage>
        ) : null}
        <Button
          type="button"
          disabled={leaveAction.busy}
          onClick={() => void confirmLeave()}
        >
          {leaveAction.busy ? t("common.saving") : t("team.leave.submit")}
        </Button>
      </div>
    </Dialog>
  );
}
