import { useEffect, useRef, useState } from "react";
import { LogIn, Plus, UsersRound } from "lucide-react";
import { useSession } from "@/features/staff/hooks/useSession";
import { teamApi } from "@/features/team/api/teamApi";
import { CreateTeamDialog } from "@/features/team/components/CreateTeamDialog";
import { JoinTeamDialog } from "@/features/team/components/JoinTeamDialog";
import { TeamDetailsDialog } from "@/features/team/components/TeamDetailsDialog";
import { TeamSyncStatus } from "@/features/team/components/TeamSyncStatus";
import { useTeamSidebar } from "@/features/team/hooks/useTeamSidebar";
import type { CreateTeamResult, JoinTeamResult } from "@/features/team/types/team";
import { useI18n } from "@/shared/hooks/useI18n";
import { useModalAsyncAction } from "@/shared/hooks/useModalAsyncAction";
import { cn } from "@/shared/utils/cn";
import { Button, Dialog, StatusMessage } from "@/ui";

type Props = {
  collapsed: boolean;
};

export function TeamSidebar({ collapsed }: Props) {
  const { t } = useI18n();
  const { applySession, refresh } = useSession();
  const {
    team,
    pin,
    members,
    nearby,
    syncStatus,
    loading,
    error,
    setError,
    setPin,
    reload,
  } = useTeamSidebar();
  const [createOpen, setCreateOpen] = useState(false);
  const [joinOpen, setJoinOpen] = useState(false);
  const [leaveOpen, setLeaveOpen] = useState(false);
  const [detailsOpen, setDetailsOpen] = useState(false);
  const autoOpenedRef = useRef(false);
  const leaveAction = useModalAsyncAction(leaveOpen);

  useEffect(() => {
    if (team) {
      autoOpenedRef.current = false;
      return;
    }
    if (createOpen || joinOpen || autoOpenedRef.current || nearby.length === 0) {
      return;
    }
    autoOpenedRef.current = true;
    setJoinOpen(true);
  }, [team, nearby.length, createOpen, joinOpen]);

  async function handleCreated(result: CreateTeamResult) {
    applySession(result.session);
    setPin(result.pin);
    setDetailsOpen(true);
    await reload();
  }

  async function handleJoined(result: JoinTeamResult) {
    applySession(result.session);
    setDetailsOpen(true);
    await reload();
  }

  async function confirmLeave() {
    const done = await leaveAction.run(async () => {
      await teamApi.leave();
      await refresh();
      await reload();
    }, t("team.errors.lastAdminLeave"));
    if (done !== null) {
      setLeaveOpen(false);
      setDetailsOpen(false);
    }
  }

  function openDetails() {
    setDetailsOpen(true);
  }

  return (
    <div
      className={cn(
        "mt-4 border-t border-border pt-4",
        collapsed && "md:flex md:flex-col md:items-center md:gap-2",
      )}
    >
      {loading ? (
        <StatusMessage className={cn(collapsed && "md:hidden")}>
          {t("common.loading")}
        </StatusMessage>
      ) : null}
      {error ? (
        <StatusMessage tone="danger" className={cn("mb-2", collapsed && "md:hidden")}>
          {error}
        </StatusMessage>
      ) : null}

      {!loading && !team ? (
        <>
          <div className={cn("flex flex-col gap-2", collapsed && "md:hidden")}>
            <p className="text-sm font-medium">{t("team.title")}</p>
            <p className="text-sm text-muted">{t("team.notInTeam")}</p>
            <Button type="button" onClick={() => setCreateOpen(true)}>
              {t("team.create.submit")}
            </Button>
            <Button
              type="button"
              variant="secondary"
              onClick={() => setJoinOpen(true)}
            >
              {t("team.enter")}
            </Button>
          </div>
          {collapsed ? (
            <>
              <Button
                type="button"
                variant="secondary"
                className="hidden size-8 shrink-0 p-0 md:inline-flex"
                aria-label={t("team.create.submit")}
                title={t("team.create.submit")}
                onClick={() => setCreateOpen(true)}
              >
                <Plus className="size-4" aria-hidden />
              </Button>
              <Button
                type="button"
                variant="secondary"
                className="hidden size-8 shrink-0 p-0 md:inline-flex"
                aria-label={t("team.enter")}
                title={t("team.enter")}
                onClick={() => setJoinOpen(true)}
              >
                <LogIn className="size-4" aria-hidden />
              </Button>
            </>
          ) : null}
        </>
      ) : null}

      {!loading && team ? (
        <>
          <div className={cn("flex flex-col gap-3", collapsed && "md:hidden")}>
            <p className="truncate text-sm font-semibold">{team.name}</p>
            <TeamSyncStatus status={syncStatus} />
            {pin ? (
              <div className="flex flex-col gap-1">
                <p className="text-xs font-medium text-muted">
                  {t("team.pin.label")}
                </p>
                <p className="select-all text-2xl font-semibold tracking-widest">
                  {pin}
                </p>
                <p className="text-xs text-muted">{t("team.pin.help")}</p>
              </div>
            ) : null}
            <Button type="button" onClick={openDetails}>
              {t("team.members.open")}
            </Button>
            <Button
              type="button"
              variant="secondary"
              onClick={() => {
                setError(null);
                setLeaveOpen(true);
              }}
            >
              {t("team.leave.submit")}
            </Button>
          </div>
          {collapsed ? (
            <Button
              type="button"
              variant="secondary"
              className="hidden size-8 shrink-0 p-0 md:inline-flex"
              aria-label={t("team.members.open")}
              title={t("team.members.open")}
              onClick={openDetails}
            >
              <UsersRound className="size-4" aria-hidden />
            </Button>
          ) : null}
        </>
      ) : null}

      <CreateTeamDialog
        open={createOpen}
        onOpenChange={setCreateOpen}
        onCreated={handleCreated}
      />
      <JoinTeamDialog
        open={joinOpen}
        nearby={nearby}
        onOpenChange={setJoinOpen}
        onJoined={handleJoined}
      />
      {team ? (
        <TeamDetailsDialog
          open={detailsOpen}
          teamName={team.name}
          pin={pin}
          members={members}
          syncStatus={syncStatus}
          onOpenChange={setDetailsOpen}
          footer={
            <Button
              type="button"
              variant="secondary"
              onClick={() => {
                setDetailsOpen(false);
                setError(null);
                setLeaveOpen(true);
              }}
            >
              {t("team.leave.submit")}
            </Button>
          }
        />
      ) : null}
      <Dialog
        open={leaveOpen}
        onOpenChange={setLeaveOpen}
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
    </div>
  );
}
