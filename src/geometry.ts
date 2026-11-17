// Rectangle math for the selection, in CSS pixels.

export interface Point {
  x: number;
  y: number;
}

export interface Rect {
  x: number;
  y: number;
  w: number;
  h: number;
}

/** The eight resize handles, named by compass direction. */
export type Handle = "nw" | "n" | "ne" | "e" | "se" | "s" | "sw" | "w";

export const HANDLES: readonly Handle[] = ["nw", "n", "ne", "e", "se", "s", "sw", "w"];

/** Rect spanned by two corner points (any order). */
export function rectFromPoints(a: Point, b: Point): Rect {
  return {
    x: Math.min(a.x, b.x),
    y: Math.min(a.y, b.y),
    w: Math.abs(a.x - b.x),
    h: Math.abs(a.y - b.y),
  };
}

export function contains(r: Rect, p: Point): boolean {
  return p.x >= r.x && p.x <= r.x + r.w && p.y >= r.y && p.y <= r.y + r.h;
}

export function clampRect(r: Rect, bounds: { w: number; h: number }): Rect {
  const x = Math.max(0, Math.min(r.x, bounds.w - r.w));
  const y = Math.max(0, Math.min(r.y, bounds.h - r.h));
  return { x, y, w: Math.min(r.w, bounds.w), h: Math.min(r.h, bounds.h) };
}

export function handlePoint(r: Rect, h: Handle): Point {
  const cx = r.x + r.w / 2;
  const cy = r.y + r.h / 2;
  const right = r.x + r.w;
  const bottom = r.y + r.h;
  switch (h) {
    case "nw": return { x: r.x, y: r.y };
    case "n": return { x: cx, y: r.y };
    case "ne": return { x: right, y: r.y };
    case "e": return { x: right, y: cy };
    case "se": return { x: right, y: bottom };
    case "s": return { x: cx, y: bottom };
    case "sw": return { x: r.x, y: bottom };
    case "w": return { x: r.x, y: cy };
  }
}

export function hitHandle(r: Rect, p: Point, radius = 8): Handle | null {
  for (const h of HANDLES) {
    const hp = handlePoint(r, h);
    if (Math.abs(hp.x - p.x) <= radius && Math.abs(hp.y - p.y) <= radius) return h;
  }
  return null;
}

/** Drag handle `h` of `start` by (dx, dy); edges may cross (rect is normalised). */
export function resizeRect(start: Rect, h: Handle, dx: number, dy: number): Rect {
  let x1 = start.x;
  let y1 = start.y;
  let x2 = start.x + start.w;
  let y2 = start.y + start.h;
  if (h.includes("w")) x1 += dx;
  if (h.includes("e")) x2 += dx;
  if (h.includes("n")) y1 += dy;
  if (h.includes("s")) y2 += dy;
  return rectFromPoints({ x: x1, y: y1 }, { x: x2, y: y2 });
}

export function cursorFor(h: Handle): string {
  return h === "n" || h === "s" ? "ns-resize"
    : h === "e" || h === "w" ? "ew-resize"
    : h === "nw" || h === "se" ? "nwse-resize"
    : "nesw-resize";
}
