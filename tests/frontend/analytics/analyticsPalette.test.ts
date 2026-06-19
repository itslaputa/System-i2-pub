import { describe, expect, it } from "vitest";
import {
  getRootCategoryColor,
  normalizeRootCategoryLabel,
} from "../../../src/features/analytics/analyticsPalette";

describe("analyticsPalette", () => {
  it("normalizes punctuated root labels to canonical business labels", () => {
    expect(normalizeRootCategoryLabel("Здоровье.")).toBe("Здоровье");
    expect(normalizeRootCategoryLabel("Здоровье!")).toBe("Здоровье");
    expect(normalizeRootCategoryLabel(" Здоровье? ")).toBe("Здоровье");
  });

  it("resolves known colors for punctuated root labels", () => {
    expect(getRootCategoryColor("Здоровье.")).toEqual({
      backgroundColor: "rgba(138, 214, 158, 0.92)",
      borderColor: "rgba(98, 180, 121, 0.98)",
    });
  });
});
