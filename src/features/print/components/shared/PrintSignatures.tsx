type Props = {
  clientLabel: string;
  shopLabel: string;
  nameDateLabel: string;
};

export function PrintSignatures({
  clientLabel,
  shopLabel,
  nameDateLabel,
}: Props) {
  return (
    <section className="mt-10 grid grid-cols-2 gap-8 break-inside-avoid">
      <PrintSignatureLine label={clientLabel} nameDateLabel={nameDateLabel} />
      <PrintSignatureLine label={shopLabel} nameDateLabel={nameDateLabel} />
    </section>
  );
}

function PrintSignatureLine({
  label,
  nameDateLabel,
}: {
  label: string;
  nameDateLabel: string;
}) {
  return (
    <div>
      <p className="text-sm font-medium">{label}</p>
      <div className="mt-10 border-b border-neutral-800" />
      <p className="mt-1 text-xs text-neutral-600">{nameDateLabel}</p>
    </div>
  );
}
