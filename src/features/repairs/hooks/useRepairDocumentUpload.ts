import { open } from "@tauri-apps/plugin-dialog";
import { useState } from "react";
import { repairsApi } from "@/features/repairs/api/repairsApi";
import type { RepairDocumentType } from "@/features/repairs/types/repairDocument";
import { useI18n } from "@/shared/hooks/useI18n";

type Options = {
  repairId: string;
  onUploaded?: () => void;
};

export function useRepairDocumentUpload({ repairId, onUploaded }: Options) {
  const { t } = useI18n();
  const [uploading, setUploading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function pickAndUpload(documentType: RepairDocumentType): Promise<boolean> {
    setError(null);
    setUploading(true);
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: t("repairs.documents.upload.documentsFilter"),
            extensions: ["pdf", "png", "jpg", "jpeg"],
          },
        ],
      });
      if (selected === null || Array.isArray(selected)) {
        return false;
      }
      await repairsApi.uploadDocument(repairId, documentType, selected);
      onUploaded?.();
      return true;
    } catch (err) {
      setError(
        err instanceof Error
          ? err.message
          : t("repairs.documents.upload.uploadFailed"),
      );
      return false;
    } finally {
      setUploading(false);
    }
  }

  return { uploading, error, setError, pickAndUpload };
}
