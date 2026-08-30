import { useForm } from "@tanstack/react-form";
import { useState } from "react";
import { teamApi } from "@/features/team/api/teamApi";
import type { CreateTeamResult } from "@/features/team/types/team";
import { useI18n } from "@/shared/hooks/useI18n";
import { Button, Dialog, FormField, StatusMessage, TextField } from "@/ui";

type Props = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onCreated: (result: CreateTeamResult) => void | Promise<void>;
};

export function CreateTeamDialog({ open, onOpenChange, onCreated }: Props) {
  const { t } = useI18n();
  const [error, setError] = useState<string | null>(null);
  const [createdPin, setCreatedPin] = useState<string | null>(null);

  const form = useForm({
    defaultValues: { name: "", memberName: "" },
    onSubmit: async ({ value }) => {
      setError(null);
      try {
        const result = await teamApi.create({
          name: value.name.trim(),
          memberName: value.memberName.trim(),
        });
        setCreatedPin(result.pin);
        await onCreated(result);
      } catch (err) {
        setError(err instanceof Error ? err.message : t("team.create.submit"));
      }
    },
  });

  function handleOpenChange(next: boolean) {
    if (!next) {
      form.reset();
      setError(null);
      setCreatedPin(null);
    }
    onOpenChange(next);
  }

  return (
    <Dialog
      open={open}
      onOpenChange={handleOpenChange}
      title={t("team.create.title")}
      closeLabel={t("common.close")}
    >
      {createdPin ? (
        <div className="flex flex-col gap-3">
          <p className="select-all text-2xl font-semibold tracking-widest">
            {createdPin}
          </p>
          <StatusMessage>{t("team.create.pinShown")}</StatusMessage>
        </div>
      ) : (
        <form
          className="flex flex-col gap-3"
          onSubmit={(event) => {
            event.preventDefault();
            event.stopPropagation();
            void form.handleSubmit();
          }}
        >
          {error ? <StatusMessage tone="danger">{error}</StatusMessage> : null}
          <form.Field
            name="name"
            validators={{
              onChange: ({ value }) =>
                !value.trim() ? t("team.create.name") : undefined,
            }}
          >
            {(field) => (
              <FormField
                label={t("team.create.name")}
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
            name="memberName"
            validators={{
              onChange: ({ value }) =>
                !value.trim() ? t("team.create.memberName") : undefined,
            }}
          >
            {(field) => (
              <FormField
                label={t("team.create.memberName")}
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
          <form.Subscribe
            selector={(state) => [state.canSubmit, state.isSubmitting] as const}
          >
            {([canSubmit, isSubmitting]) => (
              <Button type="submit" disabled={!canSubmit || isSubmitting}>
                {isSubmitting ? t("common.saving") : t("team.create.submit")}
              </Button>
            )}
          </form.Subscribe>
        </form>
      )}
    </Dialog>
  );
}
