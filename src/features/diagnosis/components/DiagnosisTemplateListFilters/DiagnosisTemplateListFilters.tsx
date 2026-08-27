import { SearchField } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

export type DiagnosisTemplateListFiltersProps = {
  query: string;
  onQueryChange: (value: string) => void;
};

export function DiagnosisTemplateListFilters({
  query,
  onQueryChange,
}: DiagnosisTemplateListFiltersProps) {
  const { t } = useI18n();

  return (
    <div className="flex flex-wrap items-end gap-4">
      <SearchField
        id="diagnosis-template-search"
        label={t("diagnosis.search")}
        value={query}
        placeholder={t("diagnosis.searchPlaceholder")}
        onChange={(event) => onQueryChange(event.target.value)}
      />
    </div>
  );
}
