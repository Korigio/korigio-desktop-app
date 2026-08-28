import { useState } from "react";
import { repairsApi } from "@/features/repairs/api/repairsApi";
import type { Repair } from "@/features/repairs/types/repair";
import {
  REPAIR_DOCUMENT_TYPES,
  REPAIR_DOCUMENT_PRINT_PATHS,
  type RepairDocument,
  type RepairDocumentType,
} from "@/features/repairs/types/repairDocument";
import {
  documentCardStatus,
  documentForType,
} from "@/features/repairs/utils/repairWorkflow";
import { WorkflowCardStatusBadge } from "@/features/repairs/components/WorkflowCardStatusBadge";
import { useI18n } from "@/shared/hooks/useI18n";
import { Button, Card, LinkButton, StatusMessage } from "@/ui";

type Props = {
  repair: Repair;
  documents: RepairDocument[];
  documentsLoading?: boolean;
  onUploadClick: (documentType: RepairDocumentType) => void;
  onDocumentsChange: () => void;
};

export function RepairDocumentCard({
  repair,
  documents,
  documentsLoading = false,
  onUploadClick,
  onDocumentsChange,
}: Props) {
  const { t } = useI18n();
  const cardStatus = documentCardStatus(repair, documents);
  const [busyType, setBusyType] = useState<RepairDocumentType | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function handleView(documentType: RepairDocumentType) {
    setError(null);
    setBusyType(documentType);
    try {
      await repairsApi.openDocument(repair.id, documentType);
    } catch (err) {
      setError(
        err instanceof Error ? err.message : t("repairs.documents.openFailed"),
      );
    } finally {
      setBusyType(null);
    }
  }

  async function handleDelete(documentType: RepairDocumentType) {
    const confirmed = window.confirm(t("repairs.documents.deleteConfirm"));
    if (!confirmed) {
      return;
    }

    setError(null);
    setBusyType(documentType);
    try {
      await repairsApi.deleteDocument(repair.id, documentType);
      onDocumentsChange();
    } catch (err) {
      setError(
        err instanceof Error ? err.message : t("repairs.documents.deleteFailed"),
      );
    } finally {
      setBusyType(null);
    }
  }

  return (
    <Card
      title={t("repairs.documents.title")}
      actions={<WorkflowCardStatusBadge status={cardStatus} />}
      className={cardStatus === "disabled" ? "opacity-60" : undefined}
    >
      {error ? (
        <StatusMessage tone="danger" className="mb-3">
          {error}
        </StatusMessage>
      ) : null}

      {documentsLoading ? (
        <p className="text-sm text-muted">{t("repairs.documents.loading")}</p>
      ) : (
        <ul className="flex flex-col gap-4">
          {REPAIR_DOCUMENT_TYPES.map((documentType) => {
            const doc = documentForType(documents, documentType);
            const busy = busyType === documentType;

            return (
              <li
                key={documentType}
                className="flex flex-col gap-2 border-b border-border pb-4 last:border-b-0 last:pb-0"
              >
                <div className="flex flex-wrap items-start justify-between gap-2">
                  <div className="min-w-0">
                    <p className="text-sm font-medium text-foreground">
                      {t(`repairs.documents.types.${documentType}.label`)}
                    </p>
                    <p className="text-xs text-muted">
                      {t(`repairs.documents.types.${documentType}.description`)}
                    </p>
                  </div>
                  <LinkButton
                    variant="secondary"
                    to={REPAIR_DOCUMENT_PRINT_PATHS[documentType](repair.id)}
                  >
                    {t("repairs.documents.print")}
                  </LinkButton>
                </div>

                {doc ? (
                  <div className="flex flex-col gap-2">
                    <p className="truncate text-sm text-foreground">
                      {doc.originalFilename}
                    </p>
                    <div className="flex flex-wrap gap-2">
                      <Button
                        type="button"
                        variant="secondary"
                        disabled={busy}
                        onClick={() => void handleView(documentType)}
                      >
                        {busy
                          ? t("repairs.documents.opening")
                          : t("repairs.documents.view")}
                      </Button>
                      <Button
                        type="button"
                        variant="secondary"
                        disabled={busy}
                        onClick={() => onUploadClick(documentType)}
                      >
                        {t("repairs.documents.replace")}
                      </Button>
                      <Button
                        type="button"
                        variant="secondary"
                        disabled={busy}
                        onClick={() => void handleDelete(documentType)}
                      >
                        {t("repairs.documents.delete")}
                      </Button>
                    </div>
                  </div>
                ) : (
                  <div>
                    <p className="mb-2 text-sm text-muted">
                      {t("repairs.documents.noneUploaded")}
                    </p>
                    <Button
                      type="button"
                      disabled={cardStatus === "disabled" || busy}
                      onClick={() => onUploadClick(documentType)}
                    >
                      {t("repairs.documents.uploadButton")}
                    </Button>
                  </div>
                )}
              </li>
            );
          })}
        </ul>
      )}
    </Card>
  );
}
