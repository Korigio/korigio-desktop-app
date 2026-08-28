import { useRepairDocumentUpload } from "@/features/repairs/hooks/useRepairDocumentUpload";
import type { RepairDocumentType } from "@/features/repairs/types/repairDocument";
import { useI18n } from "@/shared/hooks/useI18n";
import { Button, Dialog, StatusMessage } from "@/ui";

type Props = {
  repairId: number;
  documentType: RepairDocumentType;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess: () => void;
};

export function UploadRepairDocumentModal({
  repairId,
  documentType,
  open: isOpen,
  onOpenChange,
  onSuccess,
}: Props) {
  const { t } = useI18n();
  const { uploading, error, pickAndUpload } = useRepairDocumentUpload({
    repairId,
  });

  async function handlePickFile() {
    const uploaded = await pickAndUpload(documentType);
    if (uploaded) {
      onSuccess();
      onOpenChange(false);
    }
  }

  return (
    <Dialog
      open={isOpen}
      onOpenChange={onOpenChange}
      title={t(`repairs.documents.types.${documentType}.uploadTitle`)}
      description={t(`repairs.documents.types.${documentType}.uploadDescription`)}
      closeLabel={t("common.close")}
    >
      <div className="flex flex-col gap-4">
        {error ? <StatusMessage tone="danger">{error}</StatusMessage> : null}
        <p className="text-sm text-muted">
          {t("repairs.documents.upload.hint")}
        </p>
        <div className="flex flex-wrap gap-2">
          <Button
            type="button"
            disabled={uploading}
            onClick={() => void handlePickFile()}
          >
            {uploading
              ? t("repairs.documents.upload.uploading")
              : t("repairs.documents.upload.chooseFile")}
          </Button>
          <Button
            type="button"
            variant="secondary"
            disabled={uploading}
            onClick={() => onOpenChange(false)}
          >
            {t("repairs.workflow.modals.cancel")}
          </Button>
        </div>
      </div>
    </Dialog>
  );
}
