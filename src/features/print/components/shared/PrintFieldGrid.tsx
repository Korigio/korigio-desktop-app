export type PrintField = {
  label: string;
  value: string;
};

type Props = {
  fields: PrintField[];
};

export function PrintFieldGrid({ fields }: Props) {
  return (
    <div className="mt-2 space-y-1 text-sm">
      {fields.map((field) => (
        <div
          key={field.label}
          className="grid grid-cols-[8rem_1fr] gap-x-3 break-inside-avoid"
        >
          <div className="text-neutral-600">{field.label}</div>
          <div>{field.value}</div>
        </div>
      ))}
    </div>
  );
}
