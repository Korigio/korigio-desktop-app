import type { StaffRole } from "@/features/staff/types/staff";
import { TeamMemberTable } from "@/features/team/components/TeamMemberTable";
import type { TeamMember } from "@/features/team/types/team";
import { Card, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  members: TeamMember[];
  canEditRoles: boolean;
  roleBusy: boolean;
  roleError: string | null;
  onChangeRole: (id: string, role: StaffRole) => void;
};

export function TeamMembersCard({
  members,
  canEditRoles,
  roleBusy,
  roleError,
  onChangeRole,
}: Props) {
  const { t } = useI18n();

  return (
    <Card title={t("team.members.title")}>
      <div className="flex flex-col gap-2">
        {roleError ? (
          <StatusMessage tone="danger">{roleError}</StatusMessage>
        ) : null}
        <TeamMemberTable
          members={members}
          canEditRoles={canEditRoles}
          roleBusy={roleBusy}
          onChangeRole={onChangeRole}
        />
        <StatusMessage>{t("team.members.gigsHint")}</StatusMessage>
      </div>
    </Card>
  );
}
