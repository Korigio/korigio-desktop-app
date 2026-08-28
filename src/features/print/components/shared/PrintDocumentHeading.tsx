type Props = {
  title: string;
  repairNumberLabel: string;
  repairNumber: string;
};

export function PrintDocumentHeading({
  title,
  repairNumberLabel,
  repairNumber,
}: Props) {
  return (
    <div className="mt-4 flex flex-wrap items-baseline justify-between gap-2">
      <h1 className="text-xl font-medium">{title}</h1>
      <p className="text-sm">
        <span className="font-medium">{repairNumberLabel}:</span> {repairNumber}
      </p>
    </div>
  );
}
