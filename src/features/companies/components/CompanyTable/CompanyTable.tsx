import {
  flexRender,
  getCoreRowModel,
  useReactTable,
  type ColumnDef,
} from "@tanstack/react-table";
import { Link } from "react-router";
import { companyLabel, type Company } from "@/features/companies/types/company";
import { Button, DataTable } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  companies: Company[];
  onArchive: (company: Company) => void;
  onUnarchive: (company: Company) => void;
  onSetDefault: (company: Company) => void;
};

export function CompanyTable({
  companies,
  onArchive,
  onUnarchive,
  onSetDefault,
}: Props) {
  const { t } = useI18n();

  const columns: ColumnDef<Company>[] = [
    {
      id: "name",
      header: t("companies.fields.legalName"),
      cell: ({ row }) => (
        <Link
          className="font-medium text-primary hover:underline"
          to={`/companies/${row.original.id}`}
        >
          {companyLabel(row.original)}
        </Link>
      ),
    },
    {
      accessorKey: "taxId",
      header: t("companies.fields.taxId"),
      cell: ({ getValue }) => getValue<string | null>() ?? "—",
    },
    {
      accessorKey: "phone",
      header: t("companies.fields.phone"),
      cell: ({ getValue }) => getValue<string | null>() ?? "—",
    },
    {
      id: "default",
      header: t("companies.fields.default"),
      cell: ({ row }) =>
        row.original.isDefault ? t("companies.status.default") : "—",
    },
    {
      id: "status",
      header: t("companies.fields.status"),
      cell: ({ row }) =>
        row.original.archivedAt
          ? t("companies.status.archived")
          : t("companies.status.active"),
    },
    {
      id: "actions",
      header: t("common.actions"),
      cell: ({ row }) => {
        const company = row.original;
        return (
          <div className="flex flex-wrap gap-2">
            {!company.archivedAt && !company.isDefault ? (
              <Button
                type="button"
                variant="secondary"
                className="px-2 py-1 text-xs"
                onClick={() => void onSetDefault(company)}
              >
                {t("companies.actions.setDefault")}
              </Button>
            ) : null}
            <Button
              type="button"
              variant="secondary"
              className="px-2 py-1 text-xs"
              onClick={() =>
                company.archivedAt
                  ? void onUnarchive(company)
                  : void onArchive(company)
              }
            >
              {company.archivedAt
                ? t("companies.actions.unarchive")
                : t("companies.actions.archive")}
            </Button>
          </div>
        );
      },
    },
  ];

  const table = useReactTable({
    data: companies,
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
            {t("companies.empty")}
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
