export type PrintField = {
  label: string;
  value: string;
};

type Props = {
  fields: PrintField[];
  /** Use `narrow` for company header-style grids. */
  variant?: "standard" | "narrow";
};

const GRID_CLASS = {
  standard: "mt-2 grid grid-cols-[8rem_1fr] gap-x-3 gap-y-1 text-sm",
  narrow: "mt-2 grid grid-cols-[6rem_1fr] gap-x-3 gap-y-0.5 text-sm",
} as const;

export function PrintFieldGrid({ fields, variant = "standard" }: Props) {
  return (
    <dl className={GRID_CLASS[variant]}>
      {fields.map((field) => (
        <PrintFieldRow key={field.label} label={field.label} value={field.value} />
      ))}
    </dl>
  );
}

function PrintFieldRow({ label, value }: PrintField) {
  return (
    <>
      <dt>{label}</dt>
      <dd>{value}</dd>
    </>
  );
}
