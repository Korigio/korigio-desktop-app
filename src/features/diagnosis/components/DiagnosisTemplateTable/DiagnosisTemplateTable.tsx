import {
  flexRender,
  getCoreRowModel,
  useReactTable,
  type ColumnDef,
} from "@tanstack/react-table";
import { Link } from "react-router";
import type { DiagnosisTemplate } from "@/features/diagnosis/types/diagnosis";
import { Button, DataTable, LinkButton } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  templates: DiagnosisTemplate[];
  onDelete: (template: DiagnosisTemplate) => void;
  deletingId?: string | null;
};

export function DiagnosisTemplateTable({
  templates,
  onDelete,
  deletingId = null,
}: Props) {
  const { t } = useI18n();

  const columns: ColumnDef<DiagnosisTemplate>[] = [
    {
      accessorKey: "name",
      header: t("diagnosis.fields.name"),
      cell: ({ row }) => (
        <Link
          className="font-medium text-primary hover:underline"
          to={`/diagnosis-templates/${row.original.id}/edit`}
        >
          {row.original.name}
        </Link>
      ),
    },
    {
      id: "itemCount",
      header: t("diagnosis.fields.items"),
      cell: ({ row }) => row.original.body.items.length,
    },
    {
      accessorKey: "updatedAt",
      header: t("diagnosis.fields.updatedAt"),
      cell: ({ getValue }) => getValue<string>(),
    },
    {
      id: "actions",
      header: t("common.actions"),
      cell: ({ row }) => {
        const template = row.original;
        return (
          <div className="flex gap-2">
            <LinkButton
              to={`/diagnosis-templates/${template.id}/edit`}
              variant="secondary"
              className="px-2 py-1 text-xs"
            >
              {t("diagnosis.actions.edit")}
            </LinkButton>
            <Button
              type="button"
              variant="secondary"
              className="px-2 py-1 text-xs"
              disabled={deletingId === template.id}
              onClick={() => void onDelete(template)}
            >
              {t("diagnosis.actions.delete")}
            </Button>
          </div>
        );
      },
    },
  ];

  const table = useReactTable({
    data: templates,
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
            {t("diagnosis.empty")}
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
