import {
  useEffect,
  useId,
  useRef,
  useState,
  type KeyboardEvent,
  type RefObject,
} from "react";
import { X } from "lucide-react";
import { FormField } from "@/ui/molecules/FormField";
import { TextField } from "@/ui/atoms/TextField";
import { cn } from "@/shared/utils/cn";

export type SearchComboboxProps<T> = {
  label: string;
  query: string;
  onQueryChange: (value: string) => void;
  items: T[];
  renderItem: (item: T) => string;
  getItemKey: (item: T) => string | number;
  onSelect: (item: T) => void;
  selectedLabel?: string | null;
  onClearSelection?: () => void;
  clearLabel?: string;
  loading?: boolean;
  emptyMessage?: string;
  disabled?: boolean;
  inputRef?: RefObject<HTMLInputElement | null>;
  footerAction?: {
    label: string;
    onClick: () => void;
  };
  className?: string;
};

export function SearchCombobox<T>({
  label,
  query,
  onQueryChange,
  items,
  renderItem,
  getItemKey,
  onSelect,
  selectedLabel = null,
  onClearSelection,
  clearLabel,
  loading = false,
  emptyMessage,
  disabled = false,
  inputRef,
  footerAction,
  className,
}: SearchComboboxProps<T>) {
  const listId = useId();
  const containerRef = useRef<HTMLDivElement>(null);
  const [open, setOpen] = useState(false);
  const [highlightIndex, setHighlightIndex] = useState(0);

  const displayValue = selectedLabel ?? query;
  const showList = open && !disabled && !selectedLabel;
  const showClear = Boolean(selectedLabel && onClearSelection);

  useEffect(() => {
    setHighlightIndex(0);
  }, [items, query]);

  useEffect(() => {
    const onPointerDown = (event: MouseEvent) => {
      if (!containerRef.current?.contains(event.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("pointerdown", onPointerDown);
    return () => document.removeEventListener("pointerdown", onPointerDown);
  }, []);

  const selectItem = (item: T) => {
    onSelect(item);
    setOpen(false);
  };

  const clearSelection = () => {
    onClearSelection?.();
    setOpen(false);
  };

  const onKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (selectedLabel) {
      if (event.key === "Backspace" || event.key === "Delete") {
        event.preventDefault();
        onClearSelection?.();
        setOpen(false);
      }
      return;
    }

    if (event.key === "ArrowDown") {
      event.preventDefault();
      setOpen(true);
      setHighlightIndex((current) =>
        items.length === 0 ? 0 : Math.min(current + 1, items.length - 1),
      );
      return;
    }

    if (event.key === "ArrowUp") {
      event.preventDefault();
      setHighlightIndex((current) => Math.max(current - 1, 0));
      return;
    }

    if (event.key === "Enter") {
      event.preventDefault();
      if (items[highlightIndex]) {
        selectItem(items[highlightIndex]);
      } else if (footerAction && items.length === 0 && query.trim()) {
        footerAction.onClick();
      }
      return;
    }

    if (event.key === "Escape") {
      event.preventDefault();
      setOpen(false);
    }
  };

  return (
    <div ref={containerRef} className={cn("relative", className)}>
      <FormField label={label} htmlFor={listId}>
        <div className="relative">
          <TextField
            ref={inputRef}
            id={listId}
            type="search"
            role="combobox"
            aria-expanded={showList}
            aria-controls={`${listId}-listbox`}
            aria-autocomplete="list"
            autoComplete="off"
            disabled={disabled}
            value={displayValue}
            className={showClear ? "pr-10" : undefined}
            onClick={() => {
              if (!disabled && !selectedLabel) {
                setOpen(true);
              }
            }}
            onChange={(event) => {
              if (selectedLabel) {
                return;
              }
              onQueryChange(event.target.value);
              setOpen(true);
            }}
            onKeyDown={onKeyDown}
          />
          {showClear ? (
            <button
              type="button"
              className={cn(
                "absolute right-1.5 top-1/2 -translate-y-1/2",
                "inline-flex size-7 items-center justify-center rounded-md text-muted",
                "hover:bg-background hover:text-foreground",
                "focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary",
                "disabled:cursor-not-allowed disabled:opacity-50",
              )}
              aria-label={clearLabel}
              title={clearLabel}
              disabled={disabled}
              onClick={(event) => {
                event.preventDefault();
                clearSelection();
              }}
            >
              <X className="size-4" aria-hidden />
            </button>
          ) : null}
        </div>
      </FormField>

      {showList ? (
        <ul
          id={`${listId}-listbox`}
          role="listbox"
          className="absolute z-20 mt-1 max-h-56 w-full overflow-auto rounded-md border border-border bg-surface py-1 shadow-sm"
        >
          {loading ? (
            <li className="px-3 py-2 text-sm text-muted">…</li>
          ) : items.length === 0 ? (
            <li className="px-3 py-2 text-sm text-muted">
              {emptyMessage ?? "—"}
            </li>
          ) : (
            items.map((item, index) => (
              <li key={getItemKey(item)}>
                <button
                  type="button"
                  role="option"
                  aria-selected={index === highlightIndex}
                  className={cn(
                    "w-full px-3 py-2 text-left text-sm",
                    index === highlightIndex
                      ? "bg-primary text-primary-foreground"
                      : "text-foreground hover:bg-background",
                  )}
                  onMouseEnter={() => setHighlightIndex(index)}
                  onClick={() => selectItem(item)}
                >
                  {renderItem(item)}
                </button>
              </li>
            ))
          )}
          {footerAction && query.trim() ? (
            <li className="border-t border-border">
              <button
                type="button"
                className="w-full px-3 py-2 text-left text-sm text-primary hover:bg-background"
                onClick={() => {
                  footerAction.onClick();
                  setOpen(false);
                }}
              >
                {footerAction.label}
              </button>
            </li>
          ) : null}
        </ul>
      ) : null}
    </div>
  );
}
