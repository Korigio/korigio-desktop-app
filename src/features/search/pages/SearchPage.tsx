import { SearchResults } from "@/features/search/components/SearchResults";
import { useGlobalSearch } from "@/features/search/hooks/useGlobalSearch";
import { Page, PageHeader, SearchField } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

export function SearchPage() {
  const { t } = useI18n();
  const search = useGlobalSearch();

  return (
    <Page>
      <PageHeader
        title={t("search.title")}
        description={t("search.subtitle")}
      />

      <SearchField
        id="global-search"
        label={t("nav.search")}
        value={search.query}
        placeholder={t("search.placeholder")}
        onChange={(event) => search.setQuery(event.target.value)}
      />

      <SearchResults
        result={search.result}
        loading={search.loading}
        error={search.error}
        hasQuery={search.debouncedQuery.trim().length > 0}
      />
    </Page>
  );
}
