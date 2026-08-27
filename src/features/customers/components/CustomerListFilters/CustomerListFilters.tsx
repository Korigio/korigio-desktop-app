import { CheckboxField, SearchField } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

export type CustomerListFiltersProps = {
  query: string;
  onQueryChange: (value: string) => void;
  includeArchived: boolean;
  onIncludeArchivedChange: (value: boolean) => void;
};

export function CustomerListFilters({
  query,
  onQueryChange,
  includeArchived,
  onIncludeArchivedChange,
}: CustomerListFiltersProps) {
  const { t } = useI18n();

  return (
    <div className="flex flex-wrap items-end gap-4">
      <SearchField
        id="customer-search"
        label={t("customers.search")}
        value={query}
        placeholder={t("customers.searchPlaceholder")}
        onChange={(event) => onQueryChange(event.target.value)}
      />
      <CheckboxField
        id="customer-include-archived"
        label={t("customers.includeArchived")}
        checked={includeArchived}
        onChange={(event) => onIncludeArchivedChange(event.target.checked)}
      />
    </div>
  );
}
