import { CheckboxField, SearchField } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

export type DeviceListFiltersProps = {
  query: string;
  onQueryChange: (value: string) => void;
  includeArchived: boolean;
  onIncludeArchivedChange: (value: boolean) => void;
};

export function DeviceListFilters({
  query,
  onQueryChange,
  includeArchived,
  onIncludeArchivedChange,
}: DeviceListFiltersProps) {
  const { t } = useI18n();

  return (
    <div className="flex flex-wrap items-end gap-4">
      <SearchField
        id="device-search"
        label={t("devices.search")}
        value={query}
        placeholder={t("devices.searchPlaceholder")}
        onChange={(event) => onQueryChange(event.target.value)}
      />
      <CheckboxField
        id="device-include-archived"
        label={t("devices.includeArchived")}
        checked={includeArchived}
        onChange={(event) => onIncludeArchivedChange(event.target.checked)}
      />
    </div>
  );
}
