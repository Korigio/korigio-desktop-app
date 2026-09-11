import {
  flexRender,
  getCoreRowModel,
  useReactTable,
  type ColumnDef,
} from "@tanstack/react-table";
import { Link } from "react-router";
import type { RepairListItem } from "@/features/repairs/types/repair";
import {
  repairWorkflowActionKey,
  workflowActionHref,
} from "@/features/repairs/utils/repairWorkflow";
import { DataTable } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  repairs: RepairListItem[];
  showCustomerLink?: boolean;
  showDeviceLink?: boolean;
};

export function RepairTable({
  repairs,
  showCustomerLink = true,
  showDeviceLink = true,
}: Props) {
  const { t } = useI18n();

  const columns: ColumnDef<RepairListItem>[] = [
    {
      accessorKey: "repairNumber",
      header: t("repairs.fields.repairNumber"),
      cell: ({ row }) => (
        <Link
          className="font-medium text-primary hover:underline"
          to={`/repairs/${row.original.id}`}
        >
          {row.original.repairNumber}
        </Link>
      ),
    },
    {
      accessorKey: "status",
      header: t("repairs.fields.status"),
      cell: ({ row }) => t(`repairs.status.${row.original.status}`),
    },
    ...(showCustomerLink
      ? [
          {
            id: "customer",
            header: t("repairs.fields.customer"),
            cell: ({ row }: { row: { original: RepairListItem } }) => (
              <Link
                className="text-primary hover:underline"
                to={`/customers/${row.original.customerId}`}
              >
                {row.original.customerName}
              </Link>
            ),
          } satisfies ColumnDef<RepairListItem>,
        ]
      : []),
    ...(showDeviceLink
      ? [
          {
            id: "device",
            header: t("repairs.fields.device"),
            cell: ({ row }: { row: { original: RepairListItem } }) => (
              <Link
                className="text-primary hover:underline"
                to={`/devices/${row.original.deviceId}`}
              >
                {row.original.deviceName}
              </Link>
            ),
          } satisfies ColumnDef<RepairListItem>,
        ]
      : []),
    {
      accessorKey: "receivedAt",
      header: t("repairs.fields.receivedAt"),
      cell: ({ getValue }) => getValue<string>() ?? "—",
    },
    {
      id: "actions",
      header: t("repairs.fields.actions"),
      cell: ({ row }) => {
        const actionKey = repairWorkflowActionKey(row.original);
        if (!actionKey) {
          return "—";
        }
        const label = t(`repairs.workflow.actions.${actionKey}`);
        return (
          <Link
            className="font-medium text-primary hover:underline"
            to={workflowActionHref(row.original.id, actionKey)}
          >
            {label}
          </Link>
        );
      },
    },
  ];

  const table = useReactTable({
    data: repairs,
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
                : flexRender(
                    header.column.columnDef.header,
                    header.getContext(),
                  )}
            </th>
          ))}
        </tr>
      ))}
    >
      {table.getRowModel().rows.length === 0 ? (
        <tr>
          <td className="px-3 py-6 text-muted" colSpan={columns.length}>
            {t("repairs.empty")}
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
