import { useCallback, useEffect, useRef, useState } from "react";

export type UseFlowWizardOptions<T extends string> = {
  steps: readonly T[];
  initialStep?: T;
  onEnter?: Partial<Record<T, () => void>>;
};

export function useFlowWizard<T extends string>({
  steps,
  initialStep,
  onEnter,
}: UseFlowWizardOptions<T>) {
  const initialIndex = (() => {
    if (initialStep === undefined) {
      return 0;
    }
    const found = steps.indexOf(initialStep);
    return found >= 0 ? found : 0;
  })();

  const [index, setIndex] = useState(initialIndex);
  const onEnterRef = useRef(onEnter);
  onEnterRef.current = onEnter;

  const clampedIndex = Math.min(Math.max(index, 0), Math.max(steps.length - 1, 0));
  const stepId = steps[clampedIndex] as T;
  const isFirst = clampedIndex <= 0;
  const isLast = clampedIndex >= steps.length - 1;

  useEffect(() => {
    onEnterRef.current?.[stepId]?.();
  }, [stepId]);

  const goTo = useCallback(
    (id: T) => {
      const nextIndex = steps.indexOf(id);
      if (nextIndex >= 0) {
        setIndex(nextIndex);
      }
    },
    [steps],
  );

  const next = useCallback(() => {
    setIndex((current) => Math.min(current + 1, steps.length - 1));
  }, [steps.length]);

  const back = useCallback(() => {
    setIndex((current) => Math.max(current - 1, 0));
  }, []);

  const reset = useCallback(() => {
    setIndex(initialIndex);
  }, [initialIndex]);

  return {
    steps,
    index: clampedIndex,
    stepId,
    isFirst,
    isLast,
    next,
    back,
    goTo,
    reset,
  };
}
