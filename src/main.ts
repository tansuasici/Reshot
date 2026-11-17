// Overlay entry point: shows the frozen frame, lets the user select a region,
// annotate it, then copy or save it. All coordinates are CSS pixels except at
// export, which crops the original frame in physical pixels.

import { drawAnnotation, FONT, LINE_HEIGHT, type Annotation, type Tool } from "./annotations";
import {
  clampRect,
  contains,
  cursorFor,
  handlePoint,
  hitHandle,
  HANDLES,
  rectFromPoints,
  resizeRect,
  type Handle,
  type Point,
  type Rect,
} from "./geometry";
import * as ipc from "./ipc";
import { ICONS } from "./icons";

const COLORS = ["#ff3b30", "#ff9500", "#ffcc00", "#34c759", "#0a84ff", "#af52de", "#111111", "#ffffff"];
const ACCENT = "#0a84ff";
const MIN_SELECTION = 4;
const GAP = 8;

type Drag =
  | { kind: "none" }
  | { kind: "select"; origin: Point }
  | { kind: "move"; origin: Point; start: Rect }
  | { kind: "resize"; handle: Handle; origin: Point; start: Rect }
  | { kind: "draw"; annotation: Annotation };

interface State {
  frame: ipc.Frame | null;
  selection: Rect | null;
  annotations: Annotation[];
  tool: Tool | null;
  color: string;
  drag: Drag;
  textAt: Point | null;
  busy: boolean;
}

const state: State = {
  frame: null,
  selection: null,
  annotations: [],
  tool: null,
  color: COLORS[0],
  drag: { kind: "none" },
  textAt: null,
  busy: false,
};

const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;
const frameCanvas = $<HTMLCanvasElement>("frame");
const uiCanvas = $<HTMLCanvasElement>("ui");
const toolsBar = $<HTMLDivElement>("tools");
const actionsBar = $<HTMLDivElement>("actions");
const palette = $<HTMLDivElement>("palette");
const textInput = $<HTMLTextAreaElement>("text-input");
const toast = $<HTMLDivElement>("toast");
const frameCtx = frameCanvas.getContext("2d") as CanvasRenderingContext2D;
const ui = uiCanvas.getContext("2d") as CanvasRenderingContext2D;

/** Physical pixels per CSS pixel, derived from the frame itself (not DPR). */
const scale = () => (state.frame ? state.frame.width / window.innerWidth : devicePixelRatio);

// ---------------------------------------------------------------- rendering

function resizeUiCanvas(): void {
  const dpr = devicePixelRatio;
  uiCanvas.width = Math.round(window.innerWidth * dpr);
  uiCanvas.height = Math.round(window.innerHeight * dpr);
}

function render(): void {
  const dpr = devicePixelRatio;
  ui.setTransform(dpr, 0, 0, dpr, 0, 0);
  ui.clearRect(0, 0, window.innerWidth, window.innerHeight);
  ui.fillStyle = "rgba(0, 0, 0, 0.45)";
  ui.fillRect(0, 0, window.innerWidth, window.innerHeight);

  const sel = state.selection;
  if (!sel) return;

  ui.clearRect(sel.x, sel.y, sel.w, sel.h);
  ui.save();
  ui.beginPath();
  ui.rect(sel.x, sel.y, sel.w, sel.h);
  ui.clip();
  for (const a of state.annotations) drawAnnotation(ui, a);
  if (state.drag.kind === "draw") drawAnnotation(ui, state.drag.annotation);
  ui.restore();

  ui.strokeStyle = ACCENT;
  ui.lineWidth = 1;
  ui.setLineDash([5, 4]);
  ui.strokeRect(sel.x + 0.5, sel.y + 0.5, sel.w - 1, sel.h - 1);
  ui.setLineDash([]);

  for (const h of HANDLES) {
    const p = handlePoint(sel, h);
    ui.fillStyle = "#fff";
    ui.strokeStyle = ACCENT;
    ui.fillRect(p.x - 4, p.y - 4, 8, 8);
    ui.strokeRect(p.x - 3.5, p.y - 3.5, 7, 7);
  }

  drawSizeLabel(sel);
}

function drawSizeLabel(sel: Rect): void {
  const s = scale();
  const text = `${Math.round(sel.w * s)} × ${Math.round(sel.h * s)}`;
  ui.font = '500 12px -apple-system, "Segoe UI", system-ui, sans-serif';
  const w = ui.measureText(text).width + 12;
  const h = 20;
  const x = Math.min(sel.x, window.innerWidth - w);
  const y = sel.y - h - 4 >= 0 ? sel.y - h - 4 : sel.y + 4;
  ui.fillStyle = "rgba(20, 20, 22, 0.85)";
  ui.beginPath();
  ui.roundRect(x, y, w, h, 4);
  ui.fill();
  ui.fillStyle = "#fff";
  ui.textBaseline = "middle";
  ui.fillText(text, x + 6, y + h / 2);
}

// ---------------------------------------------------------------- toolbars

const TOOL_BUTTONS: { tool: Tool; title: string; key: string }[] = [
  { tool: "pen", title: "Pen (P)", key: "p" },
  { tool: "line", title: "Line (L)", key: "l" },
  { tool: "arrow", title: "Arrow (A)", key: "a" },
  { tool: "rect", title: "Rectangle (R)", key: "r" },
  { tool: "marker", title: "Marker (M)", key: "m" },
  { tool: "text", title: "Text (T)", key: "t" },
];

const isMac = navigator.platform.toLowerCase().includes("mac");
const mod = isMac ? "⌘" : "Ctrl+";

function button(icon: string, title: string, onClick: () => void): HTMLButtonElement {
  const b = document.createElement("button");
  b.type = "button";
  b.title = title;
  b.innerHTML = icon;
  b.addEventListener("pointerdown", (e) => e.stopPropagation());
  b.addEventListener("click", (e) => {
    e.stopPropagation();
    onClick();
  });
  return b;
}

const toolButtons = new Map<Tool, HTMLButtonElement>();
let colorButton: HTMLButtonElement;

function buildToolbars(): void {
  for (const t of TOOL_BUTTONS) {
    const b = button(ICONS[t.tool], t.title, () => selectTool(state.tool === t.tool ? null : t.tool));
    toolButtons.set(t.tool, b);
    toolsBar.append(b);
  }
  colorButton = button('<span class="swatch"></span>', "Color", togglePalette);
  toolsBar.append(colorButton, button(ICONS.undo, `Undo (${mod}Z)`, undo));

  actionsBar.append(
    button(ICONS.copy, `Copy (${mod}C / Enter)`, () => void copy()),
    button(ICONS.save, `Save (${mod}S)`, () => void save()),
    button(ICONS.close, "Close (Esc)", () => void close()),
  );

  for (const c of COLORS) {
    const b = button(`<span class="swatch" style="background:${c}"></span>`, c, () => {
      state.color = c;
      palette.hidden = true;
      syncToolbar();
    });
    palette.append(b);
  }
  palette.addEventListener("pointerdown", (e) => e.stopPropagation());
  syncToolbar();
}

function syncToolbar(): void {
  for (const [tool, b] of toolButtons) b.classList.toggle("active", state.tool === tool);
  colorButton.querySelector<HTMLSpanElement>(".swatch")!.style.background = state.color;
}

function selectTool(tool: Tool | null): void {
  commitText();
  state.tool = tool;
  syncToolbar();
}

function togglePalette(): void {
  palette.hidden = !palette.hidden;
  if (palette.hidden) return;
  const r = colorButton.getBoundingClientRect();
  const pw = palette.offsetWidth;
  const left = r.right + GAP + pw <= window.innerWidth ? r.right + GAP : r.left - GAP - pw;
  palette.style.left = `${left}px`;
  palette.style.top = `${Math.min(r.top, window.innerHeight - palette.offsetHeight - GAP)}px`;
}

/** Lightshot layout: tools to the right of the selection, actions below it. */
function placeToolbars(): void {
  const sel = state.selection;
  const show = sel !== null && state.drag.kind === "none";
  toolsBar.hidden = !show;
  actionsBar.hidden = !show;
  if (!show) {
    palette.hidden = true;
    return;
  }
  const W = window.innerWidth;
  const H = window.innerHeight;
  const right = sel.x + sel.w;
  const bottom = sel.y + sel.h;

  const tw = toolsBar.offsetWidth;
  const th = toolsBar.offsetHeight;
  let tx = right + GAP;
  if (tx + tw > W) tx = sel.x - GAP - tw;
  if (tx < 0) tx = right - tw - GAP;
  const ty = clamp(bottom - th, GAP, H - th - GAP);
  toolsBar.style.left = `${Math.max(GAP, tx)}px`;
  toolsBar.style.top = `${ty}px`;

  const aw = actionsBar.offsetWidth;
  const ah = actionsBar.offsetHeight;
  let ay = bottom + GAP;
  if (ay + ah > H) ay = sel.y - GAP - ah;
  if (ay < 0) ay = bottom - ah - GAP;
  const ax = clamp(right - aw, GAP, W - aw - GAP);
  actionsBar.style.left = `${ax}px`;
  actionsBar.style.top = `${Math.max(GAP, ay)}px`;
}

const clamp = (v: number, lo: number, hi: number) => Math.max(lo, Math.min(v, hi));

// ---------------------------------------------------------------- pointer

function pointOf(e: PointerEvent): Point {
  return { x: clamp(e.clientX, 0, window.innerWidth), y: clamp(e.clientY, 0, window.innerHeight) };
}

uiCanvas.addEventListener("pointerdown", (e) => {
  if (e.button !== 0 || state.busy) return;
  palette.hidden = true;
  const p = pointOf(e);
  if (state.textAt) {
    commitText();
    return;
  }
  uiCanvas.setPointerCapture(e.pointerId);
  const sel = state.selection;
  const handle = sel ? hitHandle(sel, p) : null;

  if (sel && handle) {
    state.drag = { kind: "resize", handle, origin: p, start: { ...sel } };
  } else if (sel && contains(sel, p) && state.tool === "text") {
    startText(p);
  } else if (sel && contains(sel, p) && state.tool && state.tool !== "text") {
    state.drag = { kind: "draw", annotation: newAnnotation(state.tool, p) };
  } else if (sel && contains(sel, p)) {
    state.drag = { kind: "move", origin: p, start: { ...sel } };
  } else {
    state.drag = { kind: "select", origin: p };
    state.selection = { x: p.x, y: p.y, w: 0, h: 0 };
  }
  placeToolbars();
  render();
});

uiCanvas.addEventListener("pointermove", (e) => {
  const p = pointOf(e);
  const d = state.drag;
  switch (d.kind) {
    case "none":
      updateCursor(p);
      return;
    case "select":
      state.selection = rectFromPoints(d.origin, p);
      break;
    case "move":
      state.selection = clampRect(
        { ...d.start, x: d.start.x + p.x - d.origin.x, y: d.start.y + p.y - d.origin.y },
        { w: window.innerWidth, h: window.innerHeight },
      );
      break;
    case "resize":
      state.selection = resizeRect(d.start, d.handle, p.x - d.origin.x, p.y - d.origin.y);
      break;
    case "draw":
      extendAnnotation(d.annotation, p, e.shiftKey);
      break;
  }
  render();
});

const endDrag = () => {
  const d = state.drag;
  if (d.kind === "none") return;
  if (d.kind === "draw" && isMeaningful(d.annotation)) state.annotations.push(d.annotation);
  if (d.kind === "select" && state.selection && (state.selection.w < MIN_SELECTION || state.selection.h < MIN_SELECTION)) {
    state.selection = null;
  }
  state.drag = { kind: "none" };
  placeToolbars();
  render();
};
uiCanvas.addEventListener("pointerup", endDrag);
uiCanvas.addEventListener("pointercancel", endDrag);

function updateCursor(p: Point): void {
  const sel = state.selection;
  const handle = sel ? hitHandle(sel, p) : null;
  uiCanvas.style.cursor = handle
    ? cursorFor(handle)
    : sel && contains(sel, p)
      ? state.tool === "text" ? "text" : state.tool ? "crosshair" : "move"
      : "crosshair";
}

function newAnnotation(tool: Exclude<Tool, "text">, p: Point): Annotation {
  if (tool === "pen" || tool === "marker") return { kind: tool, color: state.color, points: [p] };
  return { kind: tool, color: state.color, from: p, to: p };
}

function extendAnnotation(a: Annotation, p: Point, constrain: boolean): void {
  if ("points" in a) {
    a.points.push(p);
  } else if ("from" in a) {
    a.to = constrain ? snap(a.from, p, a.kind === "rect") : p;
  }
}

/** Shift: 45° steps for lines/arrows, squares for rectangles. */
function snap(from: Point, to: Point, square: boolean): Point {
  const dx = to.x - from.x;
  const dy = to.y - from.y;
  if (square) {
    const s = Math.max(Math.abs(dx), Math.abs(dy));
    return { x: from.x + Math.sign(dx) * s, y: from.y + Math.sign(dy) * s };
  }
  const angle = Math.round(Math.atan2(dy, dx) / (Math.PI / 4)) * (Math.PI / 4);
  const len = Math.hypot(dx, dy);
  return { x: from.x + Math.cos(angle) * len, y: from.y + Math.sin(angle) * len };
}

function isMeaningful(a: Annotation): boolean {
  if ("points" in a) return a.points.length > 0;
  if ("text" in a) return a.text.trim().length > 0;
  return Math.hypot(a.to.x - a.from.x, a.to.y - a.from.y) > 2;
}

// ---------------------------------------------------------------- text tool

function startText(p: Point): void {
  state.textAt = p;
  textInput.value = "";
  textInput.style.left = `${p.x}px`;
  textInput.style.top = `${p.y}px`;
  textInput.style.color = state.color;
  textInput.style.font = FONT;
  textInput.style.lineHeight = `${LINE_HEIGHT}px`;
  textInput.hidden = false;
  fitTextInput();
  requestAnimationFrame(() => textInput.focus());
}

function fitTextInput(): void {
  ui.font = FONT;
  const lines = textInput.value.split("\n");
  const width = Math.max(40, ...lines.map((l) => ui.measureText(l).width)) + 12;
  textInput.style.width = `${width}px`;
  textInput.style.height = `${lines.length * LINE_HEIGHT + 4}px`;
}
textInput.addEventListener("input", fitTextInput);
textInput.addEventListener("pointerdown", (e) => e.stopPropagation());

function commitText(): void {
  if (!state.textAt) return;
  const text = textInput.value;
  if (text.trim()) state.annotations.push({ kind: "text", color: state.color, at: state.textAt, text });
  cancelText();
  render();
}

function cancelText(): void {
  state.textAt = null;
  textInput.hidden = true;
  textInput.blur();
}

// ---------------------------------------------------------------- actions

function undo(): void {
  if (state.textAt) {
    cancelText();
    return;
  }
  state.annotations.pop();
  render();
}

/** The selection (or whole screen) with annotations, at physical resolution. */
function exportImage(): ipc.RgbaImage | null {
  const frame = state.frame;
  if (!frame) return null;
  const s = scale();
  const sel = state.selection ?? { x: 0, y: 0, w: window.innerWidth, h: window.innerHeight };
  const x = clamp(Math.round(sel.x * s), 0, frame.width - 1);
  const y = clamp(Math.round(sel.y * s), 0, frame.height - 1);
  const w = clamp(Math.round(sel.w * s), 1, frame.width - x);
  const h = clamp(Math.round(sel.h * s), 1, frame.height - y);

  const canvas = document.createElement("canvas");
  canvas.width = w;
  canvas.height = h;
  const ctx = canvas.getContext("2d", { willReadFrequently: true }) as CanvasRenderingContext2D;
  ctx.drawImage(frameCanvas, x, y, w, h, 0, 0, w, h);
  ctx.setTransform(s, 0, 0, s, -x, -y);
  for (const a of state.annotations) drawAnnotation(ctx, a);
  return { width: w, height: h, pixels: ctx.getImageData(0, 0, w, h).data };
}

async function runAction(action: (img: ipc.RgbaImage) => Promise<boolean>): Promise<void> {
  if (state.busy) return;
  commitText();
  const img = exportImage();
  if (!img) return;
  state.busy = true;
  try {
    if (await action(img)) await close();
  } catch (e) {
    showToast(String(e));
  } finally {
    state.busy = false;
  }
}

const copy = () => runAction(async (img) => (await ipc.copyImage(img), true));
const save = () => runAction((img) => ipc.saveImage(img, defaultFileName()));

function defaultFileName(): string {
  const d = new Date();
  const p = (n: number) => String(n).padStart(2, "0");
  return `Reshot ${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())} at ${p(d.getHours())}.${p(d.getMinutes())}.${p(d.getSeconds())}.png`;
}

async function close(): Promise<void> {
  reset();
  await ipc.closeOverlay();
}

/** Resolve after the browser has presented a frame (bounded, in case rAF stalls). */
function nextPaint(): Promise<void> {
  return new Promise((resolve) => {
    const timer = window.setTimeout(resolve, 100);
    requestAnimationFrame(() =>
      requestAnimationFrame(() => {
        window.clearTimeout(timer);
        resolve();
      }),
    );
  });
}

function reset(): void {
  cancelText();
  state.frame = null;
  state.selection = null;
  state.annotations = [];
  state.drag = { kind: "none" };
  frameCanvas.width = 0; // free the frame's memory
  frameCanvas.height = 0;
  placeToolbars();
  ui.clearRect(0, 0, uiCanvas.width, uiCanvas.height);
}

function showToast(message: string): void {
  toast.textContent = message;
  toast.hidden = false;
  setTimeout(() => (toast.hidden = true), 3000);
}

// ---------------------------------------------------------------- keyboard

window.addEventListener("keydown", (e) => {
  const cmd = isMac ? e.metaKey : e.ctrlKey;
  if (state.textAt) {
    if (e.key === "Escape") cancelText();
    else if (e.key === "Enter" && !e.shiftKey) commitText();
    else return; // let the textarea handle typing
    e.preventDefault();
    return;
  }
  const key = e.key.toLowerCase();
  if (e.key === "Escape") {
    if (!palette.hidden) palette.hidden = true;
    else void close();
  } else if (e.key === "Enter" || (cmd && key === "c")) {
    void copy();
  } else if (cmd && key === "s") {
    void save();
  } else if (cmd && key === "z") {
    undo();
  } else if (!cmd && !e.altKey && state.selection) {
    const t = TOOL_BUTTONS.find((b) => b.key === key);
    if (!t) return;
    selectTool(state.tool === t.tool ? null : t.tool);
  } else {
    return;
  }
  e.preventDefault();
});

window.addEventListener("resize", () => {
  if (!state.frame) return;
  resizeUiCanvas();
  placeToolbars();
  render();
});

// ---------------------------------------------------------------- boot

/** Longest we wait for the window to reach its new size before showing anyway. */
const VIEWPORT_TIMEOUT_MS = 500;

/**
 * The backend moves/resizes the hidden window asynchronously (possibly onto a
 * monitor with another scale factor), so the capture event can arrive while
 * the viewport still has its old size/DPR. Wait until it matches the frame.
 */
function viewportReady(meta: ipc.FrameMeta): Promise<void> {
  const matches = () =>
    Math.abs(window.innerWidth - meta.cssWidth) <= 1 &&
    Math.abs(window.innerHeight - meta.cssHeight) <= 1 &&
    Math.abs(window.innerWidth * devicePixelRatio - meta.width) <= 2;
  // Poll with timers, not rAF: a hidden window may never get animation frames.
  return new Promise((resolve) => {
    const started = performance.now();
    let timer = 0;
    const check = () => {
      if (!matches() && performance.now() - started < VIEWPORT_TIMEOUT_MS) {
        timer = window.setTimeout(check, 10);
        return;
      }
      window.clearTimeout(timer);
      window.removeEventListener("resize", check);
      resolve();
    };
    window.addEventListener("resize", check);
    check();
  });
}

async function onCapture(meta: ipc.FrameMeta): Promise<void> {
  reset();
  const [frame] = await Promise.all([ipc.getFrame(meta), viewportReady(meta)]);
  state.frame = frame;
  frameCanvas.width = frame.width;
  frameCanvas.height = frame.height;
  frameCtx.putImageData(new ImageData(frame.pixels as Uint8ClampedArray<ArrayBuffer>, frame.width, frame.height), 0, 0);
  resizeUiCanvas();
  render();
  // WebKit does not paint a hidden window: show it transparent, wait until the
  // new frame is composited, then make it visible — no stale/blank flash.
  await ipc.showOverlay();
  window.focus();
  await nextPaint();
  await ipc.revealOverlay();
}

buildToolbars();
void ipc
  .onCapture((meta) => {
    onCapture(meta).catch((e) => showToast(String(e)));
  })
  .then(() => ipc.overlayReady());
