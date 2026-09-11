import { useEffect, useState, type ReactNode } from "react";
import { useNavigate } from "react-router";
import type { Repair } from "@/features/repairs/types/repair";
import type {
  RepairDocument,
  RepairDocumentType,
} from "@/features/repairs/types/repairDocument";
import { useRepairDocumentUpload } from "@/features/repairs/hooks/useRepairDocumentUpload";
import { collectedAtOrToday } from "@/features/repairs/utils/isoDate";
import { documentForType } from "@/features/repairs/utils/repairWorkflow";
import { useModalAsyncAction } from "@/shared/hooks/useModalAsyncAction";
import { useI18n } from "@/shared/hooks/useI18n";
import {
  Button,
  Dialog,
  FormField,
  LinkButton,
  StatusMessage,
  TextField,
} from "@/ui";

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

/** Optional handover/pickup date + warranty for the summary signed-document path. */
export type ConfirmSignedDocumentHandover = {
  label: string;
  requiredMessage: string;
  failedMessage: string;
  warrantyYearsLabel: string;
  warrantyYearsRequiredMessage: string;
  warrantyYearsInvalidMessage: string;
  record: (
    repairId: string,
    collectedAt: string,
    warrantyYears: number,
  ) => Promise<Repair>;
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
  /** When set, shows date + warranty fields and persists before print/confirm. */
  handover?: ConfirmSignedDocumentHandover;
  /** Optional extra content after the hint (unused by stock wrappers). */
  children?: ReactNode;
};

const WARRANTY_YEARS_MAX = 10;

function defaultWarrantyYearsInput(warrantyYears: number | null): string {
  return String(warrantyYears ?? 1);
}

/** Required integer in 0..10. */
function parseWarrantyYearsInput(
  raw: string,
): { ok: true; value: number } | { ok: false; reason: "required" | "invalid" } {
  const trimmed = raw.trim();
  if (!trimmed) return { ok: false, reason: "required" };
  if (!/^\d+$/.test(trimmed)) return { ok: false, reason: "invalid" };
  const value = Number(trimmed);
  if (!Number.isInteger(value) || value < 0 || value > WARRANTY_YEARS_MAX) {
    return { ok: false, reason: "invalid" };
  }
  return { ok: true, value };
}

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
  handover,
  children,
}: Props) {
  const { t } = useI18n();
  const navigate = useNavigate();
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
  const [handoverDate, setHandoverDate] = useState(() =>
    collectedAtOrToday(repair.collectedAt),
  );
  const [warrantyYearsInput, setWarrantyYearsInput] = useState(() =>
    defaultWarrantyYearsInput(repair.warrantyYears),
  );

  useEffect(() => {
    if (!isOpen || !handover) return;
    setHandoverDate(collectedAtOrToday(repair.collectedAt));
    setWarrantyYearsInput(defaultWarrantyYearsInput(repair.warrantyYears));
  }, [isOpen, handover, repair.collectedAt, repair.warrantyYears]);

  const document = documentForType(documents, documentType);
  const hasUploaded = Boolean(document);
  const displayError = error ?? uploadError;
  const disabled = busy || uploading;

  async function persistHandoverIfNeeded(): Promise<boolean> {
    if (!handover) return true;
    if (!handoverDate.trim()) {
      setError(handover.requiredMessage);
      return false;
    }
    const parsed = parseWarrantyYearsInput(warrantyYearsInput);
    if (!parsed.ok) {
      setError(
        parsed.reason === "required"
          ? handover.warrantyYearsRequiredMessage
          : handover.warrantyYearsInvalidMessage,
      );
      return false;
    }
    const updated = await run(
      () => handover.record(repair.id, handoverDate, parsed.value),
      handover.failedMessage,
    );
    if (!updated) return false;
    onSuccess(updated);
    return true;
  }

  async function handlePrint() {
    setUploadError(null);
    const ok = await persistHandoverIfNeeded();
    if (ok) navigate(printTo);
  }

  async function handleConfirm() {
    setUploadError(null);
    if (handover) {
      const ok = await persistHandoverIfNeeded();
      if (!ok) return;
    }
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

        {handover ? (
          <>
            <FormField label={handover.label} htmlFor="summary-handover-date">
              <TextField
                id="summary-handover-date"
                name="handoverDate"
                type="date"
                value={handoverDate}
                disabled={disabled}
                onChange={(event) => setHandoverDate(event.target.value)}
              />
            </FormField>
            <FormField
              label={handover.warrantyYearsLabel}
              htmlFor="summary-warranty-years"
            >
              <TextField
                id="summary-warranty-years"
                name="warrantyYears"
                type="number"
                inputMode="numeric"
                min={0}
                max={WARRANTY_YEARS_MAX}
                step={1}
                value={warrantyYearsInput}
                disabled={disabled}
                onChange={(event) => setWarrantyYearsInput(event.target.value)}
              />
            </FormField>
          </>
        ) : null}

        {children}

        <div className="flex flex-wrap gap-2">
          {handover ? (
            <Button
              type="button"
              variant="secondary"
              disabled={disabled}
              onClick={() => void handlePrint()}
            >
              {messages.printButton}
            </Button>
          ) : (
            <LinkButton to={printTo} variant="secondary">
              {messages.printButton}
            </LinkButton>
          )}
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
