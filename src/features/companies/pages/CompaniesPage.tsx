import { CompanyListFilters } from "@/features/companies/components/CompanyListFilters";
import { CompanyTable } from "@/features/companies/components/CompanyTable";
import { NewCompanyButton } from "@/features/companies/components/NewCompanyButton";
import { useCompanyList } from "@/features/companies/hooks/useCompanyList";
import { Page, PageHeader, PaginationBar, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

export function CompaniesPage() {
  const { t } = useI18n();
  const list = useCompanyList();
  const totalPages = list.result
    ? Math.max(1, Math.ceil(list.result.total / list.result.pageSize))
    : 1;

  return (
    <Page>
      <PageHeader
        title={t("companies.title")}
        description={t("companies.subtitle")}
        actions={<NewCompanyButton />}
      />

      <CompanyListFilters
        query={list.query}
        onQueryChange={list.setQuery}
        includeArchived={list.includeArchived}
        onIncludeArchivedChange={list.setIncludeArchived}
      />

      {list.error ? (
        <StatusMessage tone="danger">{list.error}</StatusMessage>
      ) : null}

      {list.loading ? (
        <StatusMessage>{t("common.loading")}</StatusMessage>
      ) : (
        <CompanyTable
          companies={list.result?.items ?? []}
          onArchive={(company) => void list.archive(company)}
          onUnarchive={(company) => void list.unarchive(company)}
          onSetDefault={(company) => void list.setDefault(company)}
        />
      )}

      <PaginationBar
        page={list.page}
        totalPages={totalPages}
        onPrevious={() => list.setPage(list.page - 1)}
        onNext={() => list.setPage(list.page + 1)}
      />
    </Page>
  );
}
