import { Card } from "@/ui";

type Props = {
  label: string;
  value: number | string;
};

export function DashboardKpiCard({ label, value }: Props) {
  return (
    <Card className="min-w-0">
      <p className="text-2xl font-semibold tabular-nums text-foreground">
        {value}
      </p>
      <p className="mt-1 text-xs text-muted">{label}</p>
    </Card>
  );
}
