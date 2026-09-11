import { forwardRef, type InputHTMLAttributes } from "react";
import { cn } from "@/shared/utils/cn";

export type TextFieldProps = InputHTMLAttributes<HTMLInputElement>;

export const TextField = forwardRef<HTMLInputElement, TextFieldProps>(
  function TextField({ className, ...props }, ref) {
    return (
      <input
        ref={ref}
        className={cn(
          "box-border h-10 w-full rounded-md border border-border bg-surface px-3 text-sm text-foreground",
          "placeholder:text-muted focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary",
          "disabled:cursor-not-allowed disabled:opacity-50",
          className,
        )}
        {...props}
      />
    );
  },
);
