/**
 * Tests for the home page's screenshot GALLERY: every entry has a named shot
 * with alt text, positive dimensions and a caption, and names are unique
 * (they are React keys and file names). Routes are outside coverage; this is
 * a guard on the data only.
 */
import { describe, expect, it } from "vitest";
import { GALLERY } from "./index";

describe("GALLERY", () => {
  it("is not empty", () => {
    expect(GALLERY.length).toBeGreaterThan(0);
  });

  it("gives every shot a name, alt text, size and caption", () => {
    for (const { shot, caption } of GALLERY) {
      expect(shot.name).toMatch(/^[a-z][a-z0-9-]*$/);
      expect(shot.alt.length).toBeGreaterThan(10);
      expect(shot.width).toBeGreaterThan(0);
      expect(shot.height).toBeGreaterThan(0);
      expect(caption.trim()).not.toBe("");
    }
  });

  it("uses each shot name once", () => {
    const names = GALLERY.map((g) => g.shot.name);
    expect(new Set(names).size).toBe(names.length);
  });
});
