import { cn } from "@/shared/utils/cn";

type Props = {
  children: string;
  className?: string;
};

export function PrintLongText({ children, className }: Props) {
  return (
    <p
      className={cn(
        "mt-2 whitespace-pre-wrap break-inside-auto text-sm leading-relaxed",
        className,
      )}
    >
      {children}
    </p>
  );
}
