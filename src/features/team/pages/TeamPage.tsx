import { useState } from "react";
import { useSession } from "@/features/staff/hooks/useSession";
import { CreateTeamDialog } from "@/features/team/components/CreateTeamDialog";
import { JoinTeamDialog } from "@/features/team/components/JoinTeamDialog";
import { TeamLeaveDialog } from "@/features/team/components/TeamLeaveDialog";
import { TeamMembersCard } from "@/features/team/components/TeamMembersCard";
import { TeamPinCard } from "@/features/team/components/TeamPinCard";
import { TeamSyncStatus } from "@/features/team/components/TeamSyncStatus";
import { useTeamPage } from "@/features/team/hooks/useTeamPage";
import type { CreateTeamResult, JoinTeamResult } from "@/features/team/types/team";
import { Button, Card, Page, PageHeader, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

export function TeamPage() {
  const { t } = useI18n();
  const { applySession, session } = useSession();
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
    changeMemberRole,
    roleError,
    roleBusy,
  } = useTeamPage();
  const canEditRoles = session?.staff.role === "admin";
  const [createOpen, setCreateOpen] = useState(false);
  const [joinOpen, setJoinOpen] = useState(false);
  const [leaveOpen, setLeaveOpen] = useState(false);

  async function handleCreated(result: CreateTeamResult) {
    applySession(result.session);
    setPin(result.pin);
    await reload();
  }

  async function handleJoined(result: JoinTeamResult) {
    applySession(result.session);
    await reload();
  }

  return (
    <Page className="max-w-5xl">
      {loading && !team ? (
        <>
          <PageHeader title={t("team.title")} />
          <StatusMessage>{t("common.loading")}</StatusMessage>
        </>
      ) : team ? (
        <>
          <PageHeader
            title={team.name}
            actions={
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
            }
          />
          {error ? (
            <StatusMessage tone="danger">{error}</StatusMessage>
          ) : null}
          <div className="flex flex-col gap-4">
            <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
              <TeamPinCard pin={pin} />
              {syncStatus ? (
                <Card>
                  <TeamSyncStatus status={syncStatus} />
                </Card>
              ) : null}
            </div>
            <TeamMembersCard
              members={members}
              canEditRoles={canEditRoles}
              roleBusy={roleBusy}
              roleError={roleError}
              onChangeRole={changeMemberRole}
            />
          </div>
        </>
      ) : (
        <>
          <PageHeader
            title={t("team.title")}
            description={t("team.notInTeam")}
            actions={
              <>
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
              </>
            }
          />
          {error ? (
            <StatusMessage tone="danger">{error}</StatusMessage>
          ) : null}
        </>
      )}

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
      <TeamLeaveDialog
        open={leaveOpen}
        onOpenChange={setLeaveOpen}
        onLeft={reload}
      />
    </Page>
  );
}
