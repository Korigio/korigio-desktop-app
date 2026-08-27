import type { ReactNode, RefObject } from "react";
import { Button } from "@/ui/atoms/Button";
import { cn } from "@/shared/utils/cn";

export type WizardNavProps = {
  backLabel?: string;
  onBack?: () => void;
  showBack?: boolean;
  primaryLabel: string;
  onPrimary: () => void;
  primaryDisabled?: boolean;
  primaryType?: "button" | "submit";
  primaryRef?: RefObject<HTMLButtonElement | null>;
  hint?: ReactNode;
  className?: string;
};

export function WizardNav({
  backLabel,
  onBack,
  showBack = Boolean(onBack),
  primaryLabel,
  onPrimary,
  primaryDisabled = false,
  primaryType = "button",
  primaryRef,
  hint,
  className,
}: WizardNavProps) {
  return (
    <div className={cn("flex flex-col gap-2", className)}>
      <div className="flex flex-wrap gap-2">
        {showBack && onBack && backLabel ? (
          <Button type="button" variant="secondary" onClick={onBack}>
            {backLabel}
          </Button>
        ) : null}
        <Button
          ref={primaryRef}
          type={primaryType}
          disabled={primaryDisabled}
          onClick={primaryType === "button" ? onPrimary : undefined}
        >
          {primaryLabel}
        </Button>
      </div>
      {hint ? <div className="text-xs text-muted">{hint}</div> : null}
    </div>
  );
}
