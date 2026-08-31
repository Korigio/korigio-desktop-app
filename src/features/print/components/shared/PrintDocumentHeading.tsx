type Props = {
  title: string;
  dateLabel: string;
  formattedDate: string;
  repairNumberLabel: string;
  repairNumber: string;
};

function labeled(label: string, value: string): string {
  const trimmed = label.replace(/:\s*$/, "");
  return `${trimmed}: ${value}`;
}

export function PrintDocumentHeading({
  title,
  dateLabel,
  formattedDate,
  repairNumberLabel,
  repairNumber,
}: Props) {
  return (
    <div className="mt-6 flex items-start justify-between gap-4">
      <h1 className="text-2xl font-semibold uppercase tracking-wide">{title}</h1>
      <div className="shrink-0 text-right text-sm leading-snug">
        <p>{labeled(dateLabel, formattedDate)}</p>
        <p>{labeled(repairNumberLabel, repairNumber)}</p>
      </div>
    </div>
  );
}
