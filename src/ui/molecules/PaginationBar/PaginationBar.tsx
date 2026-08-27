import { Button } from "@/ui/atoms/Button";
import { useI18n } from "@/shared/hooks/useI18n";

export type PaginationBarProps = {
  page: number;
  totalPages: number;
  onPrevious: () => void;
  onNext: () => void;
};

export function PaginationBar({
  page,
  totalPages,
  onPrevious,
  onNext,
}: PaginationBarProps) {
  const { t } = useI18n();

  return (
    <div className="flex items-center gap-3">
      <Button
        type="button"
        variant="secondary"
        disabled={page <= 1}
        onClick={onPrevious}
      >
        {t("common.prev")}
      </Button>
      <span className="text-sm text-muted">
        {t("common.page")} {page} / {totalPages}
      </span>
      <Button
        type="button"
        variant="secondary"
        disabled={page >= totalPages}
        onClick={onNext}
      >
        {t("common.next")}
      </Button>
    </div>
  );
}
