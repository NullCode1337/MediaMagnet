/* eslint-disable no-useless-assignment */

export type AccentKind = "hue" | "black" | "white";

export type Accent = {
  kind: AccentKind;
  hue: number;
};

export function hueToHex(h: number): string {
  const s = 1,
    l = 0.5;
  const c = (1 - Math.abs(2 * l - 1)) * s;
  const x = c * (1 - Math.abs(((h / 60) % 2) - 1));
  const m = l - c / 2;
  let r = 0,
    g = 0,
    b = 0;

  if (h < 60) {
    r = c;
    g = x;
    b = 0;
  } else if (h < 120) {
    r = x;
    g = c;
    b = 0;
  } else if (h < 180) {
    r = 0;
    g = c;
    b = x;
  } else if (h < 240) {
    r = 0;
    g = x;
    b = c;
  } else if (h < 300) {
    r = x;
    g = 0;
    b = c;
  } else {
    r = c;
    g = 0;
    b = x;
  }

  const hex = (v: number) =>
    Math.round((v + m) * 255)
      .toString(16)
      .padStart(2, "0");
  return `#${hex(r)}${hex(g)}${hex(b)}`;
}

export function accentToHex(accent: Accent): string {
  if (accent.kind === "black") return "#000000";
  if (accent.kind === "white") return "#ffffff";
  return hueToHex(accent.hue);
}

export function hexToAccent(hex: string): Accent | null {
  const m = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex.trim());
  if (!m) return null;

  const r = parseInt(m[1], 16) / 255;
  const g = parseInt(m[2], 16) / 255;
  const b = parseInt(m[3], 16) / 255;

  const max = Math.max(r, g, b),
    min = Math.min(r, g, b);

  if (max === min) {
    if (max <= 0.1) return { kind: "black", hue: 0 };
    if (max >= 0.9) return { kind: "white", hue: 0 };
    return null;
  }

  const d = max - min;
  let h = 0;
  if (max === r) h = ((g - b) / d + (g < b ? 6 : 0)) / 6;
  else if (max === g) h = ((b - r) / d + 2) / 6;
  else h = ((r - g) / d + 4) / 6;

  return { kind: "hue", hue: Math.round(h * 360) };
}
