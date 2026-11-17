// Annotation model and rendering. Coordinates are CSS pixels in screen space;
// export replays them on a scaled context, so they stay crisp on HiDPI.

import type { Point } from "./geometry";

export type Tool = "pen" | "line" | "arrow" | "rect" | "marker" | "text";

export type Annotation =
  | { kind: "pen" | "marker"; color: string; points: Point[] }
  | { kind: "line" | "arrow" | "rect"; color: string; from: Point; to: Point }
  | { kind: "text"; color: string; at: Point; text: string };

export const STROKE_WIDTH = 3;
export const MARKER_WIDTH = 18;
export const FONT_SIZE = 20;
export const FONT = `600 ${FONT_SIZE}px -apple-system, "Segoe UI", system-ui, sans-serif`;
export const LINE_HEIGHT = FONT_SIZE * 1.25;

export function drawAnnotation(ctx: CanvasRenderingContext2D, a: Annotation): void {
  ctx.save();
  ctx.strokeStyle = a.color;
  ctx.fillStyle = a.color;
  ctx.lineWidth = STROKE_WIDTH;
  ctx.lineCap = "round";
  ctx.lineJoin = "round";
  switch (a.kind) {
    case "pen":
    case "marker":
      if (a.kind === "marker") {
        ctx.globalAlpha = 0.4;
        ctx.lineWidth = MARKER_WIDTH;
        ctx.lineCap = "butt";
      }
      ctx.beginPath();
      a.points.forEach((p, i) => (i === 0 ? ctx.moveTo(p.x, p.y) : ctx.lineTo(p.x, p.y)));
      if (a.points.length === 1) ctx.lineTo(a.points[0].x + 0.1, a.points[0].y);
      ctx.stroke();
      break;
    case "line":
      ctx.beginPath();
      ctx.moveTo(a.from.x, a.from.y);
      ctx.lineTo(a.to.x, a.to.y);
      ctx.stroke();
      break;
    case "arrow":
      drawArrow(ctx, a.from, a.to);
      break;
    case "rect":
      ctx.strokeRect(
        Math.min(a.from.x, a.to.x),
        Math.min(a.from.y, a.to.y),
        Math.abs(a.to.x - a.from.x),
        Math.abs(a.to.y - a.from.y),
      );
      break;
    case "text":
      ctx.font = FONT;
      ctx.textBaseline = "top";
      a.text.split("\n").forEach((line, i) => ctx.fillText(line, a.at.x, a.at.y + i * LINE_HEIGHT));
      break;
  }
  ctx.restore();
}

function drawArrow(ctx: CanvasRenderingContext2D, from: Point, to: Point): void {
  const angle = Math.atan2(to.y - from.y, to.x - from.x);
  const length = Math.hypot(to.x - from.x, to.y - from.y);
  const head = Math.min(18, Math.max(8, length / 3));
  // Stop the shaft inside the head so the round cap doesn't poke out.
  const shaftEnd = { x: to.x - Math.cos(angle) * head * 0.6, y: to.y - Math.sin(angle) * head * 0.6 };
  ctx.beginPath();
  ctx.moveTo(from.x, from.y);
  ctx.lineTo(shaftEnd.x, shaftEnd.y);
  ctx.stroke();
  ctx.beginPath();
  ctx.moveTo(to.x, to.y);
  ctx.lineTo(to.x - head * Math.cos(angle - Math.PI / 7), to.y - head * Math.sin(angle - Math.PI / 7));
  ctx.lineTo(to.x - head * Math.cos(angle + Math.PI / 7), to.y - head * Math.sin(angle + Math.PI / 7));
  ctx.closePath();
  ctx.fill();
}
