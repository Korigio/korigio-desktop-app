import { TeamMemberTable } from "@/features/team/components/TeamMemberTable";
import type { TeamMember } from "@/features/team/types/team";
import { Card, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  members: TeamMember[];
};

export function TeamMembersCard({ members }: Props) {
  const { t } = useI18n();

  return (
    <Card title={t("team.members.title")}>
      <div className="flex flex-col gap-2">
        <TeamMemberTable members={members} />
        <StatusMessage>{t("team.members.gigsHint")}</StatusMessage>
      </div>
    </Card>
  );
}
