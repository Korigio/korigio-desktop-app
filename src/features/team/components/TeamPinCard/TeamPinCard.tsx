import { Card, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  pin: string | null;
};

export function TeamPinCard({ pin }: Props) {
  const { t } = useI18n();

  return (
    <Card title={t("team.pin.label")}>
      {pin ? (
        <div className="flex flex-col gap-1">
          <p className="select-all text-2xl font-semibold tracking-widest">
            {pin}
          </p>
          <p className="text-sm text-muted">{t("team.pin.help")}</p>
        </div>
      ) : (
        <StatusMessage>{t("team.pin.unavailable")}</StatusMessage>
      )}
    </Card>
  );
}
