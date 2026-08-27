import {
  useCallback,
  useEffect,
  useRef,
  useState,
  type RefObject,
} from "react";
import { useDebouncedValue } from "@/shared/hooks/useDebouncedValue";

type Options<T> = {
  debounceMs?: number;
  search: (query: string) => Promise<T[]>;
  getLabel: (item: T) => string;
  enabled?: boolean;
  onSelect?: (item: T) => void;
  onClear?: () => void;
};

export function useAsyncSearchCombobox<T>({
  debounceMs = 250,
  search,
  getLabel,
  enabled = true,
  onSelect,
  onClear,
}: Options<T>) {
  const inputRef = useRef<HTMLInputElement>(null);
  const [query, setQuery] = useState("");
  const debouncedQuery = useDebouncedValue(query, debounceMs);
  const [items, setItems] = useState<T[]>([]);
  const [loading, setLoading] = useState(false);
  const [selected, setSelected] = useState<T | null>(null);

  useEffect(() => {
    if (!enabled) {
      setItems([]);
      return;
    }

    let cancelled = false;
    setLoading(true);
    void search(debouncedQuery)
      .then((next) => {
        if (!cancelled) {
          setItems(next);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setItems([]);
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [debouncedQuery, enabled, search]);

  const select = useCallback(
    (item: T) => {
      setSelected(item);
      setQuery("");
      onSelect?.(item);
    },
    [onSelect],
  );

  const clear = useCallback(() => {
    setSelected(null);
    setQuery("");
    onClear?.();
  }, [onClear]);

  const reset = useCallback(() => {
    setSelected(null);
    setQuery("");
    setItems([]);
  }, []);

  const focus = useCallback(() => {
    inputRef.current?.focus();
  }, []);

  return {
    inputRef: inputRef as RefObject<HTMLInputElement | null>,
    query,
    setQuery,
    items,
    loading,
    selected,
    selectedLabel: selected ? getLabel(selected) : null,
    select,
    clear,
    reset,
    focus,
    getLabel,
    enabled,
  };
}
