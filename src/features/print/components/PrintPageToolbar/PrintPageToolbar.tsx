import { Button, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  onPrint: () => void;
  onClose: () => void;
  busy: boolean;
  error: string | null;
  closeLabel: string;
};

export function PrintPageToolbar({
  onPrint,
  onClose,
  busy,
  error,
  closeLabel,
}: Props) {
  const { t } = useI18n();

  return (
    <div className="print:hidden border-b border-border bg-surface px-4 py-3">
      <div className="flex flex-wrap items-center gap-2">
        <Button type="button" disabled={busy} onClick={onPrint}>
          {t("print.actions.print")}
        </Button>
        <Button
          type="button"
          variant="secondary"
          disabled={busy}
          onClick={onClose}
        >
          {closeLabel}
        </Button>
      </div>
      {error ? (
        <StatusMessage tone="danger" className="mt-2">
          {error}
        </StatusMessage>
      ) : null}
    </div>
  );
}
