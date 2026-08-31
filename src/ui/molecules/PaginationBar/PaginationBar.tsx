import { useId } from "react";
import { PAGE_SIZE_OPTIONS } from "@/shared/constants/pagination";
import { useI18n } from "@/shared/hooks/useI18n";
import { Button } from "@/ui/atoms/Button";
import { SelectField } from "@/ui/atoms/SelectField";

export type PaginationBarProps = {
  page: number;
  totalPages: number;
  onPrevious: () => void;
  onNext: () => void;
  pageSize: number;
  onPageSizeChange: (size: number) => void;
};

export function PaginationBar({
  page,
  totalPages,
  onPrevious,
  onNext,
  pageSize,
  onPageSizeChange,
}: PaginationBarProps) {
  const { t } = useI18n();
  const pageSizeId = useId();
  const pageSizeLabel = t("common.pageSize");

  return (
    <div className="flex flex-wrap items-center justify-between gap-3">
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
      <div className="ml-auto flex items-center gap-2">
        <label htmlFor={pageSizeId} className="text-sm text-muted">
          {pageSizeLabel}
        </label>
        <SelectField
          id={pageSizeId}
          aria-label={pageSizeLabel}
          className="w-auto min-w-20"
          value={pageSize}
          onChange={(event) => {
            onPageSizeChange(Number(event.target.value));
          }}
        >
          {PAGE_SIZE_OPTIONS.map((size) => (
            <option key={size} value={size}>
              {size}
            </option>
          ))}
        </SelectField>
      </div>
    </div>
  );
}
