import {
  flexRender,
  getCoreRowModel,
  useReactTable,
  type ColumnDef,
} from "@tanstack/react-table";
import { Link } from "react-router";
import type { Customer } from "@/features/customers/types/customer";
import { Button, DataTable } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  customers: Customer[];
  onArchive: (customer: Customer) => void;
  onUnarchive: (customer: Customer) => void;
};

export function CustomerTable({ customers, onArchive, onUnarchive }: Props) {
  const { t } = useI18n();

  const columns: ColumnDef<Customer>[] = [
    {
      accessorKey: "name",
      header: t("customers.fields.name"),
      cell: ({ row }) => (
        <Link
          className="font-medium text-primary hover:underline"
          to={`/customers/${row.original.id}`}
        >
          {row.original.name}
        </Link>
      ),
    },
    {
      accessorKey: "phone",
      header: t("customers.fields.phone"),
      cell: ({ getValue }) => getValue<string | null>() ?? "—",
    },
    {
      accessorKey: "email",
      header: t("customers.fields.email"),
      cell: ({ getValue }) => getValue<string | null>() ?? "—",
    },
    {
      id: "status",
      header: t("customers.fields.status"),
      cell: ({ row }) =>
        row.original.archivedAt
          ? t("customers.status.archived")
          : t("customers.status.active"),
    },
    {
      id: "actions",
      header: t("common.actions"),
      cell: ({ row }) => {
        const customer = row.original;
        return (
          <div className="flex gap-2">
            <Button
              type="button"
              variant="secondary"
              className="px-2 py-1 text-xs"
              onClick={() =>
                customer.archivedAt
                  ? void onUnarchive(customer)
                  : void onArchive(customer)
              }
            >
              {customer.archivedAt
                ? t("customers.actions.unarchive")
                : t("customers.actions.archive")}
            </Button>
          </div>
        );
      },
    },
  ];

  const table = useReactTable({
    data: customers,
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
            {t("customers.empty")}
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
