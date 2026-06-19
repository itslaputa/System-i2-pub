import { describe, expect, it } from "vitest";
import { normalizePickedFolderPath } from "../../../src/features/setup/setupFolderPicker";

describe("normalizePickedFolderPath", () => {
  it("keeps a selected string path", () => {
    expect(normalizePickedFolderPath("/tmp/system-i2-data")).toBe(
      "/tmp/system-i2-data",
    );
  });

  it("keeps the first path when the dialog returns an array", () => {
    expect(
      normalizePickedFolderPath([
        "/tmp/system-i2-data",
        "/tmp/another-system-i2-data",
      ]),
    ).toBe("/tmp/system-i2-data");
  });

  it("returns null for blank or canceled selections", () => {
    expect(normalizePickedFolderPath("")).toBeNull();
    expect(normalizePickedFolderPath([])).toBeNull();
    expect(normalizePickedFolderPath(null)).toBeNull();
  });
});
