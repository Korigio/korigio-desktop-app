import { SelectField, SearchField } from "@/ui";
import { REPAIR_STATUSES } from "@/features/repairs/types/repair";
import type { RepairStatus } from "@/features/repairs/types/repair";
import { useI18n } from "@/shared/hooks/useI18n";

export type RepairListFiltersProps = {
  query: string;
  onQueryChange: (value: string) => void;
  status: RepairStatus | "";
  onStatusChange: (value: RepairStatus | "") => void;
};

export function RepairListFilters({
  query,
  onQueryChange,
  status,
  onStatusChange,
}: RepairListFiltersProps) {
  const { t } = useI18n();

  return (
    <div className="flex flex-wrap items-end gap-4">
      <SearchField
        id="repair-search"
        label={t("repairs.search")}
        value={query}
        placeholder={t("repairs.searchPlaceholder")}
        onChange={(event) => onQueryChange(event.target.value)}
      />
      <div className="flex flex-col gap-1">
        <label className="text-sm font-medium" htmlFor="repair-status-filter">
          {t("repairs.fields.status")}
        </label>
        <SelectField
          id="repair-status-filter"
          name="repair-status-filter"
          value={status}
          onChange={(event) =>
            onStatusChange(event.target.value as RepairStatus | "")
          }
        >
          <option value="">{t("repairs.statusFilterAll")}</option>
          {REPAIR_STATUSES.map((code) => (
            <option key={code} value={code}>
              {t(`repairs.status.${code}`)}
            </option>
          ))}
        </SelectField>
      </div>
    </div>
  );
}
