import { useForm } from "@tanstack/react-form";
import { useEffect, useMemo, useState } from "react";
import { teamApi } from "@/features/team/api/teamApi";
import type { JoinTeamResult, NearbyTeam } from "@/features/team/types/team";
import { useI18n } from "@/shared/hooks/useI18n";
import { Button, Dialog, FormField, StatusMessage, TextField } from "@/ui";

const PIN_PATTERN = /^\d{6}$/;
const NEARBY_POLL_MS = 2000;

type Props = {
  open: boolean;
  nearby: NearbyTeam[];
  onOpenChange: (open: boolean) => void;
  onJoined: (result: JoinTeamResult) => void | Promise<void>;
};

export function JoinTeamDialog({
  open,
  nearby,
  onOpenChange,
  onJoined,
}: Props) {
  const { t } = useI18n();
  const [error, setError] = useState<string | null>(null);
  const [items, setItems] = useState<NearbyTeam[]>(nearby);
  const [selectedTeamId, setSelectedTeamId] = useState<string | null>(null);

  useEffect(() => {
    if (open) {
      setItems(nearby);
    }
  }, [nearby, open]);

  useEffect(() => {
    if (!open) {
      return;
    }
    let cancelled = false;
    async function tick() {
      try {
        const result = await teamApi.listNearby();
        if (!cancelled) {
          setItems(result.items);
        }
      } catch {
        // Keep the last successful nearby list until the next tick.
      }
    }
    void tick();
    const id = window.setInterval(() => {
      void tick();
    }, NEARBY_POLL_MS);
    return () => {
      cancelled = true;
      window.clearInterval(id);
    };
  }, [open]);

  const selected = useMemo(
    () => items.find((item) => item.teamId === selectedTeamId) ?? null,
    [items, selectedTeamId],
  );

  useEffect(() => {
    if (!open) {
      return;
    }
    if (items.length === 1) {
      setSelectedTeamId(items[0].teamId);
      return;
    }
    if (
      selectedTeamId &&
      !items.some((item) => item.teamId === selectedTeamId)
    ) {
      setSelectedTeamId(null);
    }
  }, [items, open, selectedTeamId]);

  const form = useForm({
    defaultValues: { memberName: "", pin: "" },
    onSubmit: async ({ value }) => {
      if (!selected) {
        return;
      }
      setError(null);
      try {
        const result = await teamApi.join({
          teamId: selected.teamId,
          pin: value.pin.trim(),
          memberName: value.memberName.trim(),
        });
        await onJoined(result);
        handleOpenChange(false);
      } catch (err) {
        setError(err instanceof Error ? err.message : t("team.join.submit"));
      }
    },
  });

  function handleOpenChange(next: boolean) {
    if (!next) {
      form.reset();
      setError(null);
      setSelectedTeamId(null);
    }
    onOpenChange(next);
  }

  const title = selected
    ? t("team.join.pick").replace("{name}", selected.name)
    : t("team.join.title");

  return (
    <Dialog
      open={open}
      onOpenChange={handleOpenChange}
      title={title}
      closeLabel={t("common.close")}
    >
      <div className="flex flex-col gap-3">
        {items.length === 0 ? (
          <StatusMessage>{t("team.join.nearbyEmpty")}</StatusMessage>
        ) : null}

        {items.length > 1 && !selected ? (
          <ul className="flex flex-col gap-2">
            {items.map((item) => (
              <li key={item.teamId}>
                <Button
                  type="button"
                  variant="secondary"
                  className="w-full"
                  onClick={() => setSelectedTeamId(item.teamId)}
                >
                  {t("team.join.pick").replace("{name}", item.name)}
                </Button>
              </li>
            ))}
          </ul>
        ) : null}

        {selected ? (
          <form
            className="flex flex-col gap-3"
            onSubmit={(event) => {
              event.preventDefault();
              event.stopPropagation();
              void form.handleSubmit();
            }}
          >
            {error ? (
              <StatusMessage tone="danger">{error}</StatusMessage>
            ) : null}
            <form.Field
              name="memberName"
              validators={{
                onChange: ({ value }) =>
                  !value.trim() ? t("team.join.memberName") : undefined,
              }}
            >
              {(field) => (
                <FormField
                  label={t("team.join.memberName")}
                  htmlFor={field.name}
                  error={
                    typeof field.state.meta.errors[0] === "string"
                      ? field.state.meta.errors[0]
                      : undefined
                  }
                >
                  <TextField
                    id={field.name}
                    name={field.name}
                    autoComplete="off"
                    value={field.state.value}
                    onBlur={field.handleBlur}
                    onChange={(event) => field.handleChange(event.target.value)}
                  />
                </FormField>
              )}
            </form.Field>
            <form.Field
              name="pin"
              validators={{
                onChange: ({ value }) =>
                  PIN_PATTERN.test(value.trim())
                    ? undefined
                    : t("team.join.pin"),
              }}
            >
              {(field) => (
                <FormField
                  label={t("team.join.pin")}
                  htmlFor={field.name}
                  error={
                    typeof field.state.meta.errors[0] === "string"
                      ? field.state.meta.errors[0]
                      : undefined
                  }
                >
                  <TextField
                    id={field.name}
                    name={field.name}
                    inputMode="numeric"
                    autoComplete="off"
                    maxLength={6}
                    value={field.state.value}
                    onBlur={field.handleBlur}
                    onChange={(event) =>
                      field.handleChange(event.target.value.replace(/\D/g, ""))
                    }
                  />
                </FormField>
              )}
            </form.Field>
            <form.Subscribe
              selector={(state) =>
                [state.canSubmit, state.isSubmitting] as const
              }
            >
              {([canSubmit, isSubmitting]) => (
                <Button type="submit" disabled={!canSubmit || isSubmitting}>
                  {isSubmitting ? t("common.saving") : t("team.join.submit")}
                </Button>
              )}
            </form.Subscribe>
          </form>
        ) : null}
      </div>
    </Dialog>
  );
}
