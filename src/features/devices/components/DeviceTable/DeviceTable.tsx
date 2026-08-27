import {
  flexRender,
  getCoreRowModel,
  useReactTable,
  type ColumnDef,
} from "@tanstack/react-table";
import { Link } from "react-router";
import {
  deviceLabel,
  type Device,
} from "@/features/devices/types/device";
import { Button, DataTable } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  devices: Device[];
  showCustomerLink?: boolean;
  onArchive: (device: Device) => void;
  onUnarchive: (device: Device) => void;
};

export function DeviceTable({
  devices,
  showCustomerLink = true,
  onArchive,
  onUnarchive,
}: Props) {
  const { t } = useI18n();

  const columns: ColumnDef<Device>[] = [
    {
      id: "label",
      header: t("devices.fields.device"),
      cell: ({ row }) => (
        <Link
          className="font-medium text-primary hover:underline"
          to={`/devices/${row.original.id}`}
        >
          {deviceLabel(row.original)}
        </Link>
      ),
    },
    {
      accessorKey: "deviceType",
      header: t("devices.fields.deviceType"),
      cell: ({ getValue }) => getValue<string | null>() ?? "—",
    },
    {
      accessorKey: "serialNumber",
      header: t("devices.fields.serialNumber"),
      cell: ({ getValue }) => getValue<string | null>() ?? "—",
    },
    ...(showCustomerLink
      ? [
          {
            id: "customer",
            header: t("devices.fields.customer"),
            cell: ({ row }: { row: { original: Device } }) => (
              <Link
                className="text-primary hover:underline"
                to={`/customers/${row.original.customerId}`}
              >
                #{row.original.customerId}
              </Link>
            ),
          } satisfies ColumnDef<Device>,
        ]
      : []),
    {
      id: "status",
      header: t("devices.fields.status"),
      cell: ({ row }) =>
        row.original.archivedAt
          ? t("devices.status.archived")
          : t("devices.status.active"),
    },
    {
      id: "actions",
      header: t("common.actions"),
      cell: ({ row }) => {
        const device = row.original;
        return (
          <div className="flex gap-2">
            <Button
              type="button"
              variant="secondary"
              className="px-2 py-1 text-xs"
              onClick={() =>
                device.archivedAt
                  ? void onUnarchive(device)
                  : void onArchive(device)
              }
            >
              {device.archivedAt
                ? t("devices.actions.unarchive")
                : t("devices.actions.archive")}
            </Button>
          </div>
        );
      },
    },
  ];

  const table = useReactTable({
    data: devices,
    columns,
    getCoreRowModel: getCoreRowModel(),
  });

  return (
    <DataTable
      header={table.getHeaderGroups().map((headerGroup) => (
        <tr key={headerGroup.id} className="border-b border-border">
          {headerGroup.headers.map((header) => (
            <th key={header.id} className="px-3 py-2 font-medium">
              {header.isPlaceholder
                ? null
                : flexRender(header.column.columnDef.header, header.getContext())}
            </th>
          ))}
        </tr>
      ))}
    >
      {table.getRowModel().rows.length === 0 ? (
        <tr>
          <td className="px-3 py-6 text-muted" colSpan={columns.length}>
            {t("devices.empty")}
          </td>
        </tr>
      ) : (
        table.getRowModel().rows.map((row) => (
          <tr key={row.id} className="border-b border-border last:border-b-0">
            {row.getVisibleCells().map((cell) => (
              <td key={cell.id} className="px-3 py-2 align-middle">
                {flexRender(cell.column.columnDef.cell, cell.getContext())}
              </td>
            ))}
          </tr>
        ))
      )}
    </DataTable>
  );
}
