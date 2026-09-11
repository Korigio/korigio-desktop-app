import { cn } from "@/shared/utils/cn";

export type StepperStep = {
  id: string;
  label: string;
};

export type StepperProps = {
  steps: StepperStep[];
  currentStepId: string;
  className?: string;
};

type StepState = "complete" | "current" | "upcoming";

function stepState(index: number, currentIndex: number): StepState {
  if (index < currentIndex) {
    return "complete";
  }
  if (index === currentIndex) {
    return "current";
  }
  return "upcoming";
}

export function Stepper({ steps, currentStepId, className }: StepperProps) {
  const currentIndex = Math.max(
    0,
    steps.findIndex((step) => step.id === currentStepId),
  );

  return (
    <nav aria-label="Progress" className={cn("w-full", className)}>
      <ol className="flex flex-wrap items-start gap-2 sm:gap-0">
        {steps.map((step, index) => {
          const state = stepState(index, currentIndex);
          const isLast = index === steps.length - 1;

          return (
            <li
              key={step.id}
              className={cn(
                "flex min-w-0 items-center gap-2 sm:flex-1",
                !isLast && "sm:pr-2",
              )}
            >
              <div className="flex min-w-0 items-center gap-2">
                <span
                  aria-current={state === "current" ? "step" : undefined}
                  className={cn(
                    "inline-flex size-7 shrink-0 items-center justify-center rounded-full border text-xs font-medium",
                    state === "complete" &&
                      "border-primary bg-primary text-primary-foreground",
                    state === "current" &&
                      "border-primary bg-surface text-primary",
                    state === "upcoming" &&
                      "border-border bg-surface text-muted",
                  )}
                >
                  {state === "complete" ? (
                    <span aria-hidden>✓</span>
                  ) : (
                    index + 1
                  )}
                </span>
                <span
                  className={cn(
                    "truncate text-sm",
                    state === "current" && "font-medium text-foreground",
                    state === "complete" && "text-foreground",
                    state === "upcoming" && "text-muted",
                  )}
                >
                  {step.label}
                </span>
              </div>
              {!isLast ? (
                <span
                  aria-hidden
                  className="mx-1 hidden h-px flex-1 bg-border sm:block"
                />
              ) : null}
            </li>
          );
        })}
      </ol>
    </nav>
  );
}
