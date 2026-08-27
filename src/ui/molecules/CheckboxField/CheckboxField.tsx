import type { InputHTMLAttributes } from "react";
import { cn } from "@/shared/utils/cn";

export type CheckboxFieldProps = Omit<
  InputHTMLAttributes<HTMLInputElement>,
  "type"
> & {
  label: string;
};

export function CheckboxField({
  label,
  className,
  id,
  ...props
}: CheckboxFieldProps) {
  return (
    <label
      htmlFor={id}
      className={cn("flex cursor-pointer items-center gap-2 text-sm", className)}
    >
      <input id={id} type="checkbox" className="size-4 accent-primary" {...props} />
      <span>{label}</span>
    </label>
  );
}
