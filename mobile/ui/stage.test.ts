import { describe, expect, it } from "vitest";
import { fieldLayout, fit, orientOf, STAGE } from "./stage-fit";

describe("the table's stage", () => {
  it("is landscape unless the window is narrower than 0.8 of its height", () => {
    expect(orientOf(1280, 720)).toBe("L");
    expect(orientOf(800, 450)).toBe("L");
    expect(orientOf(1024, 768)).toBe("L");
    expect(orientOf(390, 844)).toBe("P");
    expect(orientOf(768, 1024)).toBe("P");
  });

  it("scales to fit the window and centres what is left over", () => {
    // 800×450 is 0.625 of the 1280×720 stage, with no bars.
    expect(fit("L", 800, 450)).toEqual({ k: 0.625, left: 0, top: 0, h: 720 });
    // A 1440×900 window is limited by its width: 1.125, bars above and below.
    const f = fit("L", 1440, 900);
    expect(f.k).toBe(1.125);
    expect(f.left).toBe(0);
    expect(f.top).toBe((900 - STAGE.L.h * 1.125) / 2);
    // A phone's own size is 1:1.
    expect(fit("P", 390, 844)).toEqual({ k: 1, left: 0, top: 0, h: 844 });
  });

  it("scales the portrait stage by the phone's width and lets its height follow", () => {
    // A 360×640 phone: 360/390 of the stage, and 640/k (693) tall, just over the 690 floor.
    const f = fit("P", 360, 640);
    expect(f.k).toBeCloseTo(360 / 390, 5);
    expect(f.h).toBeCloseTo(640 / (360 / 390), 5);
    // A tall phone keeps 1:1 and a longer stage.
    expect(fit("P", 390, 900).h).toBe(900);
    // A tablet in portrait is limited by height: the stage is 690 tall and centred.
    const t = fit("P", 768, 1024);
    expect(t.h).toBeCloseTo(690, 5);
    expect(t.left).toBeGreaterThan(0);
  });

  it("lets a flexible landscape stage grow to the window's shape, up to 4:3", () => {
    expect(fit("L", 1920, 1080, true).h).toBe(720);
    expect(fit("L", 1440, 900, true)).toEqual({ k: 1.125, left: 0, top: 0, h: 800 });
    expect(fit("L", 1024, 768, true).h).toBe(960);
    expect(fit("L", 1000, 1000, true).h).toBe(960);
    expect(fit("L", 1440, 900).h).toBe(720); // the battle's stage stays 16:9
  });

  it("keeps landscape text at 18px stage px: 11px at 800×450", () => {
    expect(18 * fit("L", 800, 450).k).toBeGreaterThanOrEqual(11);
  });
});

describe("the shop's field", () => {
  it("fits above the tray at every flexible height, pieces never under 11% of the stage", () => {
    for (const H of [720, 760, 800, 900, 960]) {
      const f = fieldLayout(H);
      expect(f.benchTop + f.benchH, `H=${H}`).toBeLessThanOrEqual(f.trayTop);
      expect(f.lineTop).toBeGreaterThanOrEqual(f.wellsTop + f.wellsH);
      expect(f.s / H, `H=${H}`).toBeGreaterThanOrEqual(0.11);
    }
  });

  it("uses a taller stage for larger pieces", () => {
    expect(fieldLayout(960).s).toBeGreaterThan(fieldLayout(720).s);
    expect(fieldLayout(720).s).toBeGreaterThanOrEqual(84);
  });
});
