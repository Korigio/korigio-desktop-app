import { CheckboxField, SearchField } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

export type CompanyListFiltersProps = {
  query: string;
  onQueryChange: (value: string) => void;
  includeArchived: boolean;
  onIncludeArchivedChange: (value: boolean) => void;
};

export function CompanyListFilters({
  query,
  onQueryChange,
  includeArchived,
  onIncludeArchivedChange,
}: CompanyListFiltersProps) {
  const { t } = useI18n();

  return (
    <div className="flex flex-wrap items-end gap-4">
      <SearchField
        id="company-search"
        label={t("companies.search")}
        value={query}
        placeholder={t("companies.searchPlaceholder")}
        onChange={(event) => onQueryChange(event.target.value)}
      />
      <CheckboxField
        id="company-include-archived"
        label={t("companies.includeArchived")}
        checked={includeArchived}
        onChange={(event) => onIncludeArchivedChange(event.target.checked)}
      />
    </div>
  );
}
