import {
  flexRender,
  getCoreRowModel,
  useReactTable,
  type ColumnDef,
} from "@tanstack/react-table";
import type { StaffRole } from "@/features/staff/types/staff";
import { TeamMemberRoleCell } from "@/features/team/components/TeamMemberRoleCell";
import type { TeamMember } from "@/features/team/types/team";
import { useI18n } from "@/shared/hooks/useI18n";
import { DataTable } from "@/ui";

type Props = {
  members: TeamMember[];
  canEditRoles: boolean;
  roleBusy?: boolean;
  onChangeRole: (id: string, role: StaffRole) => void;
};

export function TeamMemberTable({
  members,
  canEditRoles,
  roleBusy = false,
  onChangeRole,
}: Props) {
  const { t } = useI18n();
  const adminCount = members.filter((member) => member.role === "admin").length;

  const columns: ColumnDef<TeamMember>[] = [
    {
      accessorKey: "name",
      header: t("team.members.name"),
    },
    {
      accessorKey: "role",
      header: t("team.members.role"),
      cell: ({ row }) => {
        const member = row.original;
        const isLastAdmin = adminCount === 1 && member.role === "admin";
        return (
          <TeamMemberRoleCell
            member={member}
            canEditRoles={canEditRoles}
            disabled={roleBusy || isLastAdmin}
            onChange={(role) => onChangeRole(member.id, role)}
          />
        );
      },
    },
    {
      accessorKey: "online",
      header: t("team.members.status"),
      cell: ({ getValue }) =>
        getValue<boolean>()
          ? t("team.status.online")
          : t("team.status.offline"),
    },
    {
      accessorKey: "gigCount",
      header: t("team.members.gigs"),
    },
  ];

  const table = useReactTable({
    data: members,
    columns,
    getCoreRowModel: getCoreRowModel(),
  });

  return (
    <DataTable
      className="text-xs"
      header={table.getHeaderGroups().map((headerGroup) => (
        <tr key={headerGroup.id} className="border-b border-border">
          {headerGroup.headers.map((header) => (
            <th key={header.id} className="px-2 py-1.5 font-medium">
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
          <td className="px-2 py-4 text-muted" colSpan={columns.length}>
            {t("team.members.empty")}
          </td>
        </tr>
      ) : (
        table.getRowModel().rows.map((row) => (
          <tr key={row.id} className="border-b border-border last:border-b-0">
            {row.getVisibleCells().map((cell) => (
              <td key={cell.id} className="px-2 py-1.5 align-middle">
                {flexRender(cell.column.columnDef.cell, cell.getContext())}
              </td>
            ))}
          </tr>
        ))
      )}
    </DataTable>
  );
}
