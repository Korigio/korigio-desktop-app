import {
  REPAIR_STATUSES,
  type RepairStatus,
} from "@/features/repairs/types/repair";
import { SelectField } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  value: RepairStatus;
  disabled?: boolean;
  onChange: (status: RepairStatus) => void;
};

export function RepairStatusSelect({
  value,
  disabled = false,
  onChange,
}: Props) {
  const { t } = useI18n();

  return (
    <SelectField
      aria-label={t("repairs.actions.changeStatus")}
      className="min-w-[10rem] py-1 text-xs"
      disabled={disabled}
      value={value}
      onClick={(event) => event.stopPropagation()}
      onChange={(event) => onChange(event.target.value as RepairStatus)}
    >
      {REPAIR_STATUSES.map((status) => (
        <option key={status} value={status}>
          {t(`repairs.status.${status}`)}
        </option>
      ))}
    </SelectField>
  );
}
