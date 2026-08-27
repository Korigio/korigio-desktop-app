import type { InputHTMLAttributes } from "react";
import { FormField } from "@/ui/molecules/FormField";
import { TextField } from "@/ui/atoms/TextField";
import { cn } from "@/shared/utils/cn";

export type SearchFieldProps = Omit<
  InputHTMLAttributes<HTMLInputElement>,
  "type"
> & {
  label: string;
  id: string;
  className?: string;
};

export function SearchField({
  label,
  id,
  className,
  ...inputProps
}: SearchFieldProps) {
  return (
    <FormField label={label} htmlFor={id} className={cn("min-w-64 flex-1", className)}>
      <TextField id={id} type="search" {...inputProps} />
    </FormField>
  );
}
