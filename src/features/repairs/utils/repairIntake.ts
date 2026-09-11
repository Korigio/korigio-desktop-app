import { companyLabel, type Company } from "@/features/companies/types/company";
import { customerLabel } from "@/features/customers/hooks/useCustomerSearchCombobox";
import type { Customer } from "@/features/customers/types/customer";
import { deviceLabel, type Device } from "@/features/devices/types/device";
import {
  BASE_INTAKE_STEPS,
  type IntakeStep,
  type RepairIntakeReviewItem,
} from "@/features/repairs/types/repairIntake";
import {
  formatMoneyCents,
  formatTaxRateBps,
  parseDiscountPercentToBps,
  parseMajorToCents,
  previewEstimateWithDiscount,
} from "@/features/repairs/utils/money";
import type { ShopSettings } from "@/features/settings/types/shopSettings";

type IntakeDetails = {
  reportedProblem: string;
  accessoriesReceived: string;
  deviceCondition: string;
  expectedPickupAt: string;
  notes: string;
  estimateMajor?: string;
  estimateDiscountPercent?: string;
};

type Translate = (key: string) => string;

export type ResolveIntakeEstimateResult =
  | {
      ok: true;
      estimateBaseCents: number | null;
      estimateDiscountBps: number | null;
    }
  | {
      ok: false;
      reason: "discountNeedsPrice" | "invalidPrice" | "invalidDiscount";
    };

/** Map optional intake estimate fields for `create_repair`. Empty → null. */
export function resolveIntakeEstimateFields(
  estimateMajor: string,
  estimateDiscountPercent: string,
): ResolveIntakeEstimateResult {
  const priceRaw = estimateMajor.trim();
  const discountRaw = estimateDiscountPercent.trim();

  if (!priceRaw && !discountRaw) {
    return {
      ok: true,
      estimateBaseCents: null,
      estimateDiscountBps: null,
    };
  }

  if (!priceRaw) {
    return { ok: false, reason: "discountNeedsPrice" };
  }

  const estimateBaseCents = parseMajorToCents(priceRaw);
  if (estimateBaseCents === null) {
    return { ok: false, reason: "invalidPrice" };
  }

  if (!discountRaw) {
    return {
      ok: true,
      estimateBaseCents,
      estimateDiscountBps: null,
    };
  }

  const estimateDiscountBps = parseDiscountPercentToBps(discountRaw);
  if (estimateDiscountBps === null) {
    return { ok: false, reason: "invalidDiscount" };
  }

  return {
    ok: true,
    estimateBaseCents,
    estimateDiscountBps,
  };
}

export function buildIntakeSteps(
  includeCompany: boolean,
): readonly IntakeStep[] {
  return includeCompany
    ? (["company", ...BASE_INTAKE_STEPS] as const)
    : BASE_INTAKE_STEPS;
}

export function stepAfterKnownEntities(
  customer: Customer | null,
  device: Device | null,
): Extract<IntakeStep, "customer" | "device" | "details"> {
  if (customer && device) return "details";
  if (customer) return "device";
  return "customer";
}

export function preferredCompany(companies: Company[]): Company | null {
  return companies.find((item) => item.isDefault) ?? companies[0] ?? null;
}

function buildEstimateReviewItems(
  t: Translate,
  details: IntakeDetails,
  shopSettings: ShopSettings | null,
): RepairIntakeReviewItem[] {
  const estimateMajor = details.estimateMajor?.trim() ?? "";
  const discountRaw = details.estimateDiscountPercent?.trim() ?? "";
  if (!estimateMajor && !discountRaw) {
    return [];
  }

  const listCents = estimateMajor ? parseMajorToCents(estimateMajor) : null;
  const discountBps = discountRaw ? parseDiscountPercentToBps(discountRaw) : 0;
  if (
    listCents === null ||
    (discountRaw && discountBps === null) ||
    discountBps === null
  ) {
    return [];
  }

  const currency = shopSettings?.currency ?? "EUR";
  const items: RepairIntakeReviewItem[] = [
    {
      label: t("repairs.intake.estimate.fields.listPrice").replace(
        "{currency}",
        currency,
      ),
      value: formatMoneyCents(listCents, currency),
    },
  ];

  if (discountRaw && discountBps > 0) {
    items.push({
      label: t("repairs.intake.estimate.fields.discountPercent"),
      value: `${formatTaxRateBps(discountBps)}%`,
    });
  }

  if (!shopSettings) {
    return items;
  }

  const preview = previewEstimateWithDiscount(
    listCents,
    discountBps,
    shopSettings.taxRatePercent,
  );
  if (!preview) {
    return items;
  }

  items.push(
    {
      label: t("repairs.intake.review.estimateNet"),
      value: formatMoneyCents(preview.baseCents, currency),
    },
    {
      label: t("repairs.intake.estimate.preview.tax"),
      value: formatMoneyCents(preview.taxCents, currency),
    },
    {
      label: t("repairs.intake.estimate.preview.gross"),
      value: formatMoneyCents(preview.grossCents, currency),
    },
  );

  return items;
}

export function buildRepairIntakeReviewItems(
  t: Translate,
  company: Company | null,
  customer: Customer | null,
  device: Device | null,
  details: IntakeDetails,
  shopSettings: ShopSettings | null = null,
): RepairIntakeReviewItem[] {
  return [
    {
      label: t("repairs.fields.company"),
      value: company ? companyLabel(company) : "—",
    },
    {
      label: t("repairs.fields.customer"),
      value: customer ? customerLabel(customer) : "—",
    },
    {
      label: t("repairs.fields.device"),
      value: device ? deviceLabel(device) : "—",
    },
    {
      label: t("repairs.fields.reportedProblem"),
      value: details.reportedProblem.trim() || "—",
    },
    {
      label: t("repairs.fields.accessoriesReceived"),
      value: details.accessoriesReceived.trim() || "—",
    },
    {
      label: t("repairs.fields.deviceCondition"),
      value: details.deviceCondition.trim() || "—",
    },
    {
      label: t("repairs.fields.expectedPickupAt"),
      value: details.expectedPickupAt.trim() || "—",
    },
    { label: t("repairs.fields.notes"), value: details.notes.trim() || "—" },
    ...buildEstimateReviewItems(t, details, shopSettings),
  ];
}
