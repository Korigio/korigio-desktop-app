import { useForm } from "@tanstack/react-form";
import { useCallback, useEffect, useState } from "react";
import { repairsApi } from "@/features/repairs/api/repairsApi";
import type { Repair } from "@/features/repairs/types/repair";
import { staffApi } from "@/features/staff/api/staffApi";
import type { Staff } from "@/features/staff/types/staff";
import { useI18n } from "@/shared/hooks/useI18n";
import { useSyncApplied } from "@/shared/hooks/useSyncApplied";
import { Button, Dialog, FormField, SelectField, StatusMessage } from "@/ui";

type Props = {
  repair: Repair;
  onRepairChange: (repair: Repair) => void;
};

export function RepairAssigneeBar({ repair, onRepairChange }: Props) {
  const { t } = useI18n();
  const [staff, setStaff] = useState<Staff[]>([]);
  const [activeStaff, setActiveStaff] = useState<Staff[]>([]);
  const [assignOpen, setAssignOpen] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadStaff = useCallback(async (silent: boolean) => {
    try {
      const result = await staffApi.list({ includeDeactivated: true });
      setStaff(result.items);
      setActiveStaff(result.items.filter((member) => !member.deactivatedAt));
    } catch {
      if (!silent) {
        setStaff([]);
        setActiveStaff([]);
      }
    }
  }, []);

  useEffect(() => {
    void loadStaff(false);
  }, [loadStaff]);

  useSyncApplied(() => {
    void loadStaff(true);
  });

  const assigned = staff.find(
    (member) => member.id === repair.assignedToStaffId,
  );
  const form = useForm({
    defaultValues: { staffId: repair.assignedToStaffId ?? "" },
    onSubmit: async ({ value }) => {
      setBusy(true);
      setError(null);
      try {
        const next = await repairsApi.assign(repair.id, value.staffId);
        onRepairChange(next);
        setAssignOpen(false);
      } catch (err) {
        setError(err instanceof Error ? err.message : t("repairs.assign"));
      } finally {
        setBusy(false);
      }
    },
  });

  async function takeOver() {
    setBusy(true);
    setError(null);
    try {
      const next = await repairsApi.takeOver(repair.id);
      onRepairChange(next);
    } catch (err) {
      setError(err instanceof Error ? err.message : t("repairs.takeOver"));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
      <p className="text-sm">
        {assigned
          ? t("repairs.assignedTo").replace("{name}", assigned.name)
          : t("repairs.unassigned")}
      </p>
      <div className="flex flex-wrap gap-2">
        <Button
          type="button"
          variant="secondary"
          className="px-3 py-1.5 text-xs sm:text-sm"
          disabled={busy}
          onClick={() => void takeOver()}
        >
          {t("repairs.takeOver")}
        </Button>
        <Button
          type="button"
          variant="secondary"
          className="px-3 py-1.5 text-xs sm:text-sm"
          disabled={busy}
          onClick={() => setAssignOpen(true)}
        >
          {t("repairs.assign")}
        </Button>
      </div>
      {error ? <StatusMessage tone="danger">{error}</StatusMessage> : null}

      <Dialog
        open={assignOpen}
        onOpenChange={(open) => {
          setAssignOpen(open);
          if (open) {
            form.reset();
            form.setFieldValue("staffId", repair.assignedToStaffId ?? "");
          }
        }}
        title={t("repairs.assign")}
        closeLabel={t("common.close")}
      >
        <form
          className="flex flex-col gap-3"
          onSubmit={(event) => {
            event.preventDefault();
            event.stopPropagation();
            void form.handleSubmit();
          }}
        >
          <form.Field
            name="staffId"
            validators={{
              onChange: ({ value }) =>
                !value.trim() ? t("repairs.selectAssignee") : undefined,
            }}
          >
            {(field) => (
              <FormField
                label={t("repairs.selectAssignee")}
                htmlFor={field.name}
                error={
                  typeof field.state.meta.errors[0] === "string"
                    ? field.state.meta.errors[0]
                    : undefined
                }
              >
                <SelectField
                  id={field.name}
                  name={field.name}
                  value={field.state.value}
                  onBlur={field.handleBlur}
                  onChange={(event) => field.handleChange(event.target.value)}
                >
                  <option value="">{t("repairs.selectAssignee")}</option>
                  {activeStaff.map((member) => (
                    <option key={member.id} value={member.id}>
                      {member.name}
                    </option>
                  ))}
                </SelectField>
              </FormField>
            )}
          </form.Field>
          <form.Subscribe
            selector={(state) => [state.canSubmit, state.isSubmitting] as const}
          >
            {([canSubmit, isSubmitting]) => (
              <Button
                type="submit"
                disabled={!canSubmit || isSubmitting || busy}
              >
                {isSubmitting ? t("common.saving") : t("repairs.assign")}
              </Button>
            )}
          </form.Subscribe>
        </form>
      </Dialog>
    </div>
  );
}
