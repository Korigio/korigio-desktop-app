type Props = {
  /** When set with `listPrice`, shows list → optional discount → net breakdown. */
  listPriceLabel?: string;
  listPrice?: string;
  discountPercentLabel?: string;
  discountPercent?: string;
  netLabel: string;
  net: string;
  taxRateLabel: string;
  tax: string;
  totalLabel: string;
  total: string;
};

function TotalsRow({
  label,
  value,
  strong,
}: {
  label: string;
  value: string;
  strong?: boolean;
}) {
  return (
    <div
      className={
        strong
          ? "flex justify-between gap-8 font-semibold"
          : "flex justify-between gap-8"
      }
    >
      <span>{label}</span>
      <span className="tabular-nums">{value}</span>
    </div>
  );
}

export function PrintTotals({
  listPriceLabel,
  listPrice,
  discountPercentLabel,
  discountPercent,
  netLabel,
  net,
  taxRateLabel,
  tax,
  totalLabel,
  total,
}: Props) {
  const showListBreakdown = listPrice != null && listPriceLabel != null;

  return (
    <div className="mt-8 ml-auto w-64 break-inside-avoid text-sm leading-relaxed">
      {showListBreakdown ? (
        <>
          <TotalsRow label={listPriceLabel} value={listPrice} />
          {discountPercent != null && discountPercentLabel != null ? (
            <TotalsRow label={discountPercentLabel} value={discountPercent} />
          ) : null}
        </>
      ) : null}
      <TotalsRow label={netLabel} value={net} />
      <TotalsRow label={taxRateLabel} value={tax} />
      <div className="mt-1 border-t border-[#cccccc] pt-1">
        <TotalsRow label={totalLabel} value={total} strong />
      </div>
    </div>
  );
}
