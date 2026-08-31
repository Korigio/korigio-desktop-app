import { CustomerListFilters } from "@/features/customers/components/CustomerListFilters";
import { NewCustomerButton } from "@/features/customers/components/NewCustomerButton";
import { CustomerTable } from "@/features/customers/components/CustomerTable";
import { useCustomerList } from "@/features/customers/hooks/useCustomerList";
import { Page, PageHeader, PaginationBar, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

export function CustomersPage() {
  const { t } = useI18n();
  const list = useCustomerList();
  const totalPages = list.result
    ? Math.max(1, Math.ceil(list.result.total / list.result.pageSize))
    : 1;

  return (
    <Page>
      <PageHeader
        title={t("customers.title")}
        description={t("customers.subtitle")}
        actions={<NewCustomerButton />}
      />

      <CustomerListFilters
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
        <CustomerTable
          customers={list.result?.items ?? []}
          onArchive={(customer) => void list.archive(customer)}
          onUnarchive={(customer) => void list.unarchive(customer)}
        />
      )}

      <PaginationBar
        page={list.page}
        totalPages={totalPages}
        onPrevious={() => list.setPage(list.page - 1)}
        onNext={() => list.setPage(list.page + 1)}
        pageSize={list.pageSize}
        onPageSizeChange={list.setPageSize}
      />
    </Page>
  );
}
