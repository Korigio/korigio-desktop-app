import { Link, type LinkProps } from "react-router";
import { cn } from "@/shared/utils/cn";

export type LinkButtonProps = LinkProps & {
  variant?: "primary" | "secondary";
};

const variantClass = {
  primary: "bg-primary text-primary-foreground hover:opacity-90",
  secondary:
    "border border-border bg-surface text-foreground hover:bg-background",
} as const;

export function LinkButton({
  className,
  variant = "primary",
  ...props
}: LinkButtonProps) {
  return (
    <Link
      className={cn(
        "inline-flex items-center justify-center rounded-md px-4 py-2 text-sm font-medium transition-colors",
        "focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary",
        variantClass[variant],
        className,
      )}
      {...props}
    />
  );
}
