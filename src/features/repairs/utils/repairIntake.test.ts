import { describe, expect, it } from "vitest";
import type { Company } from "@/features/companies/types/company";
import type { Customer } from "@/features/customers/types/customer";
import type { Device } from "@/features/devices/types/device";
import {
  buildIntakeSteps,
  buildRepairIntakeReviewItems,
  preferredCompany,
  resolveIntakeEstimateFields,
  stepAfterKnownEntities,
} from "./repairIntake";

const customer = { id: "customer", name: "Ada" } as Customer;
const device = {
  id: "device",
  customerId: customer.id,
  manufacturer: "Dell",
  model: "XPS",
  serialNumber: "S1",
} as Device;
const company = {
  id: "company",
  legalName: "Korigio",
  tradeName: null,
  isDefault: true,
} as Company;

describe("repair intake boundaries", () => {
  it("adds the company step only for multi-company intake", () => {
    expect(buildIntakeSteps(false)).toEqual([
      "customer",
      "device",
      "details",
      "review",
      "done",
    ]);
    expect(buildIntakeSteps(true)).toEqual([
      "company",
      "customer",
      "device",
      "details",
      "review",
      "done",
    ]);
  });

  it("routes presets to the first unknown entity", () => {
    expect(stepAfterKnownEntities(null, null)).toBe("customer");
    expect(stepAfterKnownEntities(customer, null)).toBe("device");
    expect(stepAfterKnownEntities(customer, device)).toBe("details");
  });

  it("prefers the default company and falls back safely", () => {
    const other = { ...company, id: "other", isDefault: false };
    expect(preferredCompany([other, company])).toBe(company);
    expect(preferredCompany([other])).toBe(other);
    expect(preferredCompany([])).toBeNull();
  });

  it("trims details and marks empty review values", () => {
    const items = buildRepairIntakeReviewItems(
      (key) => key,
      company,
      customer,
      device,
      {
        reportedProblem: "  broken hinge  ",
        accessoriesReceived: " ",
        deviceCondition: "ok",
        expectedPickupAt: "",
        notes: " note ",
      },
    );
    expect(items.map((item) => item.value)).toEqual([
      "Korigio",
      "Ada",
      "Dell · XPS · S1",
      "broken hinge",
      "—",
      "ok",
      "—",
      "note",
    ]);
  });

  it("appends estimate review lines when list price is set", () => {
    const items = buildRepairIntakeReviewItems(
      (key) => key,
      company,
      customer,
      device,
      {
        reportedProblem: "screen",
        accessoriesReceived: "",
        deviceCondition: "",
        expectedPickupAt: "",
        notes: "",
        estimateMajor: "100",
        estimateDiscountPercent: "10",
      },
      { taxRatePercent: "19", currency: "EUR" },
    );
    const labels = items.map((item) => item.label);
    expect(labels).toContain("repairs.intake.estimate.fields.listPrice");
    expect(labels).toContain("repairs.intake.estimate.fields.discountPercent");
    expect(labels).toContain("repairs.intake.review.estimateNet");
    expect(labels).toContain("repairs.intake.estimate.preview.tax");
    expect(labels).toContain("repairs.intake.estimate.preview.gross");
    expect(
      items.find((item) => item.label === "repairs.intake.review.estimateNet")
        ?.value,
    ).toMatch(/90/);
  });

  it("resolves intake estimate fields for create_repair", () => {
    expect(resolveIntakeEstimateFields("", "")).toEqual({
      ok: true,
      estimateBaseCents: null,
      estimateDiscountBps: null,
    });
    expect(resolveIntakeEstimateFields("120.50", "")).toEqual({
      ok: true,
      estimateBaseCents: 12_050,
      estimateDiscountBps: null,
    });
    expect(resolveIntakeEstimateFields("100", "10")).toEqual({
      ok: true,
      estimateBaseCents: 10_000,
      estimateDiscountBps: 1_000,
    });
    expect(resolveIntakeEstimateFields("", "10")).toEqual({
      ok: false,
      reason: "discountNeedsPrice",
    });
    expect(resolveIntakeEstimateFields("abc", "")).toEqual({
      ok: false,
      reason: "invalidPrice",
    });
    expect(resolveIntakeEstimateFields("100", "101")).toEqual({
      ok: false,
      reason: "invalidDiscount",
    });
  });
});
