import { DiagnosisTemplateListFilters } from "@/features/diagnosis/components/DiagnosisTemplateListFilters";
import { DiagnosisTemplateTable } from "@/features/diagnosis/components/DiagnosisTemplateTable";
import { NewDiagnosisTemplateButton } from "@/features/diagnosis/components/NewDiagnosisTemplateButton";
import { useDiagnosisTemplateList } from "@/features/diagnosis/hooks/useDiagnosisTemplateList";
import { Page, PageHeader, PaginationBar, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";
import { useState } from "react";

export function DiagnosisTemplatesPage() {
  const { t } = useI18n();
  const list = useDiagnosisTemplateList();
  const [deletingId, setDeletingId] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const totalPages = list.result
    ? Math.max(1, Math.ceil(list.result.total / list.result.pageSize))
    : 1;

  return (
    <Page>
      <PageHeader
        title={t("diagnosis.title")}
        description={t("diagnosis.subtitle")}
        actions={<NewDiagnosisTemplateButton />}
      />

      <DiagnosisTemplateListFilters
        query={list.query}
        onQueryChange={list.setQuery}
      />

      {list.error ? (
        <StatusMessage tone="danger">{list.error}</StatusMessage>
      ) : null}
      {actionError ? (
        <StatusMessage tone="danger">{actionError}</StatusMessage>
      ) : null}

      {list.loading ? (
        <StatusMessage>{t("common.loading")}</StatusMessage>
      ) : (
        <DiagnosisTemplateTable
          templates={list.result?.items ?? []}
          deletingId={deletingId}
          onDelete={(template) => {
            setActionError(null);
            setDeletingId(template.id);
            void list
              .remove(template)
              .catch((err: unknown) => {
                setActionError(
                  err instanceof Error
                    ? err.message
                    : t("diagnosis.deleteFailed"),
                );
              })
              .finally(() => setDeletingId(null));
          }}
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
