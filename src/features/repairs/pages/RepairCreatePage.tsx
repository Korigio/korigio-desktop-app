import { useEffect, useState } from "react";
import { companiesApi } from "@/features/companies/api/companiesApi";
import { RepairForm } from "@/features/repairs/components/RepairForm";
import { useRepairForm } from "@/features/repairs/hooks/useRepairForm";
import { LinkButton, Page, PageHeader, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";
import { useSearchParams } from "react-router";

function parseId(raw: string | null): number | undefined {
  const id = Number(raw);
  return Number.isFinite(id) && id > 0 ? id : undefined;
}

export function RepairCreatePage() {
  const { t } = useI18n();
  const [params] = useSearchParams();
  const defaultCustomerId = parseId(params.get("customerId"));
  const defaultDeviceId = parseId(params.get("deviceId"));

  const [defaultCompanyId, setDefaultCompanyId] = useState<number | undefined>();
  const [companyGate, setCompanyGate] = useState<"loading" | "ready" | "empty">(
    "loading",
  );
  const [companyError, setCompanyError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    void companiesApi
      .list({ includeArchived: false, page: 1, pageSize: 100 })
      .then((result) => {
        if (cancelled) {
          return;
        }
        if (result.items.length === 0) {
          setCompanyGate("empty");
          return;
        }
        const preferred =
          result.items.find((item) => item.isDefault) ?? result.items[0];
        setDefaultCompanyId(preferred?.id);
        setCompanyGate("ready");
      })
      .catch((err: unknown) => {
        if (cancelled) {
          return;
        }
        setCompanyError(
          err instanceof Error ? err.message : "Failed to load companies",
        );
        setCompanyGate("empty");
      });
    return () => {
      cancelled = true;
    };
  }, []);

  if (companyGate === "loading") {
    return (
      <Page>
        <StatusMessage>{t("common.loading")}</StatusMessage>
      </Page>
    );
  }

  if (companyGate === "empty" || !defaultCompanyId) {
    return (
      <Page>
        <PageHeader
          title={t("repairs.createTitle")}
          description={t("repairs.createSubtitle")}
        />
        {companyError ? (
          <StatusMessage tone="danger">{companyError}</StatusMessage>
        ) : null}
        <StatusMessage tone="danger">
          {t("repairs.intake.noCompany")}
        </StatusMessage>
        <p className="text-sm text-muted">{t("repairs.intake.noCompanyHint")}</p>
        <div className="mt-2">
          <LinkButton to="/companies/new">
            {t("repairs.intake.noCompanyAction")}
          </LinkButton>
        </div>
      </Page>
    );
  }

  return (
    <RepairCreateForm
      defaultCustomerId={defaultCustomerId}
      defaultDeviceId={defaultDeviceId}
      defaultCompanyId={defaultCompanyId}
    />
  );
}

function RepairCreateForm({
  defaultCustomerId,
  defaultDeviceId,
  defaultCompanyId,
}: {
  defaultCustomerId?: number;
  defaultDeviceId?: number;
  defaultCompanyId: number;
}) {
  const { t } = useI18n();
  const form = useRepairForm({
    mode: "create",
    defaultCustomerId,
    defaultDeviceId,
    defaultCompanyId,
  });

  return (
    <Page>
      <PageHeader
        title={t("repairs.createTitle")}
        description={t("repairs.createSubtitle")}
      />
      <RepairForm
        form={form}
        submitLabel={t("repairs.actions.create")}
        lockCustomer={Boolean(defaultCustomerId)}
        lockDevice={Boolean(defaultDeviceId)}
      />
    </Page>
  );
}
