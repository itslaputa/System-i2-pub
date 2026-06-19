import { describe, expect, it } from "vitest";
import {
  formatTaskEntryDate,
  parseTaskDurationToMinutes,
} from "../../../src/features/task-manager/taskManagerFormatting";

describe("taskManagerFormatting", () => {
  it("formats task entry dates into compact russian form", () => {
    expect(formatTaskEntryDate("2026-03-13")).toBe("13мар 2026");
    expect(formatTaskEntryDate("bad-date")).toBe("bad-date");
  });

  it("parses plain minutes and hours.minutes inputs", () => {
    expect(parseTaskDurationToMinutes("90")).toBe(90);
    expect(parseTaskDurationToMinutes("1.30")).toBe(90);
    expect(parseTaskDurationToMinutes(" 2.05 ")).toBe(125);
  });

  it("rejects invalid duration strings", () => {
    expect(parseTaskDurationToMinutes("")).toBeNull();
    expect(parseTaskDurationToMinutes("1.")).toBeNull();
    expect(parseTaskDurationToMinutes("1.2.3")).toBeNull();
    expect(parseTaskDurationToMinutes("abc")).toBeNull();
  });
});
