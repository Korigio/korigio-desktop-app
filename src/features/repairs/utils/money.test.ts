import { describe, expect, it } from "vitest";
import {
  netCentsAfterDiscount,
  parseDiscountPercentToBps,
  parseMajorToCents,
  previewEstimate,
  previewEstimateWithDiscount,
  taxCentsFromBase,
} from "./money";

describe("money helpers", () => {
  it("parses major units to cents", () => {
    expect(parseMajorToCents("120.50")).toBe(12_050);
    expect(parseMajorToCents("100")).toBe(10_000);
    expect(parseMajorToCents("100.5")).toBe(10_050);
    expect(parseMajorToCents("")).toBeNull();
    expect(parseMajorToCents("12.345")).toBeNull();
  });

  it("parses discount percent to basis points", () => {
    expect(parseDiscountPercentToBps("10")).toBe(1_000);
    expect(parseDiscountPercentToBps("10.5")).toBe(1_050);
    expect(parseDiscountPercentToBps("0")).toBe(0);
    expect(parseDiscountPercentToBps("100")).toBe(10_000);
    expect(parseDiscountPercentToBps("100.01")).toBeNull();
    expect(parseDiscountPercentToBps("")).toBeNull();
  });

  it("computes tax cents with round half up", () => {
    expect(taxCentsFromBase(10_000, 1_900)).toBe(1_900);
    expect(taxCentsFromBase(1, 1_900)).toBe(0);
    expect(taxCentsFromBase(3, 1_900)).toBe(1);
    expect(taxCentsFromBase(5, 1_000)).toBe(1);
  });

  it("computes net after discount with round half up", () => {
    expect(netCentsAfterDiscount(10_000, 1_000)).toBe(9_000);
    expect(netCentsAfterDiscount(10_000, 0)).toBe(10_000);
    expect(netCentsAfterDiscount(10_000, 10_000)).toBe(0);
    expect(netCentsAfterDiscount(3, 5_000)).toBe(2);
    expect(netCentsAfterDiscount(1, 1_000)).toBe(1);
    expect(netCentsAfterDiscount(100, -1)).toBeNull();
    expect(netCentsAfterDiscount(100, 10_001)).toBeNull();
  });

  it("previews estimate with discount then tax", () => {
    const preview = previewEstimateWithDiscount(10_000, 1_000, "19");
    expect(preview).toEqual({
      listCents: 10_000,
      discountBps: 1_000,
      baseCents: 9_000,
      taxRateBps: 1_900,
      taxCents: 1_710,
      grossCents: 10_710,
    });
  });

  it("matches plain preview when discount is zero", () => {
    const withDiscount = previewEstimateWithDiscount(12_050, 0, "19");
    const plain = previewEstimate(12_050, "19");
    expect(withDiscount).not.toBeNull();
    expect(plain).not.toBeNull();
    expect(withDiscount?.baseCents).toBe(plain?.baseCents);
    expect(withDiscount?.taxCents).toBe(plain?.taxCents);
    expect(withDiscount?.grossCents).toBe(plain?.grossCents);
  });
});
