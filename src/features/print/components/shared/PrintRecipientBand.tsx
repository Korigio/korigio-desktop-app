import type { PrintCustomer, PrintDevice } from "@/features/print/types/printReport";
import { printFieldValue } from "@/features/print/utils/printFormat";

type Props = {
  customerTitle: string;
  deviceTitle: string;
  customer: PrintCustomer;
  device: PrintDevice;
  empty: string;
};

function optionalLines(values: Array<string | null | undefined>): string[] {
  return values
    .map((value) => value?.trim())
    .filter((value): value is string => Boolean(value && value.length > 0));
}

function BandColumn({ title, lines }: { title: string; lines: string[] }) {
  return (
    <div className="px-3 py-2">
      <p className="mb-1 text-xs font-semibold uppercase tracking-wide">
        {title}
      </p>
      <div className="text-sm leading-snug">
        {lines.map((line, index) => (
          <p key={`${index}-${line}`}>{line}</p>
        ))}
      </div>
    </div>
  );
}

export function PrintRecipientBand({
  customerTitle,
  deviceTitle,
  customer,
  device,
  empty,
}: Props) {
  return (
    <div className="mt-6 grid grid-cols-2 break-inside-avoid bg-[#f0f0f0] print:bg-[#f0f0f0]">
      <BandColumn
        title={customerTitle}
        lines={[
          printFieldValue(customer.name, empty),
          ...optionalLines([customer.phone, customer.email, customer.address]),
        ]}
      />
      <BandColumn
        title={deviceTitle}
        lines={[
          printFieldValue(device.deviceType, empty),
          ...optionalLines([
            device.manufacturer,
            device.model,
            device.serialNumber,
          ]),
        ]}
      />
    </div>
  );
}
