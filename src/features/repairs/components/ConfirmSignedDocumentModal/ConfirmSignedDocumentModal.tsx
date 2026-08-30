import type { Repair } from "@/features/repairs/types/repair";
import type { RepairDocument, RepairDocumentType } from "@/features/repairs/types/repairDocument";
import { useRepairDocumentUpload } from "@/features/repairs/hooks/useRepairDocumentUpload";
import { documentForType } from "@/features/repairs/utils/repairWorkflow";
import { useModalAsyncAction } from "@/shared/hooks/useModalAsyncAction";
import { useI18n } from "@/shared/hooks/useI18n";
import { Button, Dialog, LinkButton, StatusMessage } from "@/ui";

export type ConfirmSignedDocumentMessages = {
  title: string;
  description: string;
  hint: string;
  printButton: string;
  confirmButton: string;
  continueWithoutUpload: string;
  confirming: string;
  confirmFailed: string;
};

type Props = {
  repair: Repair;
  documents: RepairDocument[];
  documentType: RepairDocumentType;
  printTo: string;
  messages: ConfirmSignedDocumentMessages;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess: (repair: Repair) => void;
  onDocumentsChange: () => void;
  onConfirm: (repairId: string) => Promise<Repair>;
};

export function ConfirmSignedDocumentModal({
  repair,
  documents,
  documentType,
  printTo,
  messages,
  open: isOpen,
  onOpenChange,
  onSuccess,
  onDocumentsChange,
  onConfirm,
}: Props) {
  const { t } = useI18n();
  const { busy, error, setError, run } = useModalAsyncAction(isOpen);
  const {
    uploading,
    error: uploadError,
    setError: setUploadError,
    pickAndUpload,
  } = useRepairDocumentUpload({
    repairId: repair.id,
    onUploaded: onDocumentsChange,
  });

  const document = documentForType(documents, documentType);
  const hasUploaded = Boolean(document);
  const displayError = error ?? uploadError;
  const disabled = busy || uploading;

  async function handleConfirm() {
    setUploadError(null);
    const updated = await run(
      () => onConfirm(repair.id),
      messages.confirmFailed,
    );
    if (updated) {
      onSuccess(updated);
      onOpenChange(false);
    }
  }

  async function handlePickFile() {
    setError(null);
    await pickAndUpload(documentType);
  }

  const confirmLabel = hasUploaded
    ? messages.confirmButton
    : messages.continueWithoutUpload;

  return (
    <Dialog
      open={isOpen}
      onOpenChange={onOpenChange}
      title={messages.title}
      description={messages.description}
      closeLabel={t("common.close")}
    >
      <div className="flex flex-col gap-4">
        {displayError ? (
          <StatusMessage tone="danger">{displayError}</StatusMessage>
        ) : null}

        <StatusMessage tone="muted">{messages.hint}</StatusMessage>

        <div className="flex flex-wrap gap-2">
          <LinkButton to={printTo} variant="secondary">
            {messages.printButton}
          </LinkButton>
        </div>

        {document ? (
          <p className="text-sm text-foreground">{document.originalFilename}</p>
        ) : null}

        <div className="flex flex-wrap gap-2">
          <Button
            type="button"
            variant="secondary"
            disabled={disabled}
            onClick={() => void handlePickFile()}
          >
            {uploading
              ? t("repairs.documents.upload.uploading")
              : hasUploaded
                ? t("repairs.documents.replace")
                : t("repairs.documents.upload.chooseFile")}
          </Button>
        </div>

        <div className="flex flex-wrap gap-2">
          <Button
            type="button"
            disabled={disabled}
            onClick={() => void handleConfirm()}
          >
            {busy ? messages.confirming : confirmLabel}
          </Button>
          <Button
            type="button"
            variant="secondary"
            disabled={disabled}
            onClick={() => onOpenChange(false)}
          >
            {t("repairs.workflow.modals.cancel")}
          </Button>
        </div>
      </div>
    </Dialog>
  );
}
