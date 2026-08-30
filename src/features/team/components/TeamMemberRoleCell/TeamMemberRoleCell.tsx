import type { StaffRole } from "@/features/staff/types/staff";
import type { TeamMember } from "@/features/team/types/team";
import { SelectField } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

const STAFF_ROLES: StaffRole[] = ["admin", "staff"];

type Props = {
  member: TeamMember;
  canEditRoles: boolean;
  disabled?: boolean;
  onChange: (role: StaffRole) => void;
};

export function TeamMemberRoleCell({
  member,
  canEditRoles,
  disabled = false,
  onChange,
}: Props) {
  const { t } = useI18n();

  if (!canEditRoles) {
    return <>{t(`team.roles.${member.role}`)}</>;
  }

  return (
    <SelectField
      aria-label={t("team.members.role")}
      className="min-w-[10rem] py-1 text-xs"
      disabled={disabled}
      value={member.role}
      onChange={(event) => onChange(event.target.value as StaffRole)}
    >
      {STAFF_ROLES.map((role) => (
        <option key={role} value={role}>
          {t(`team.roles.${role}`)}
        </option>
      ))}
    </SelectField>
  );
}
