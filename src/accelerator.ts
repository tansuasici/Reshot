// Turn key presses into Tauri accelerator strings and back into labels.
// The strings use `KeyboardEvent.code` names (`KeyS`, `Digit9`), which the
// Rust side parses directly; `event.key` would give `(` for Shift+9.

export const isMac = navigator.userAgent.includes("Mac");

export interface KeyPress {
  code: string;
  metaKey: boolean;
  ctrlKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
}

const MODIFIER_CODES = /^(Meta|OS|Control|Alt|Shift)(Left|Right)?$/;
/** Keys that make a sensible hotkey on their own. */
const STANDALONE = /^(F\d{1,2}|PrintScreen)$/;

export type Recorded =
  | { kind: "pending" } // only modifiers so far
  | { kind: "needs-modifier" }
  | { kind: "done"; accelerator: string };

export function acceleratorFromEvent(e: KeyPress): Recorded {
  if (MODIFIER_CODES.test(e.code) || e.code === "") return { kind: "pending" };
  const parts: string[] = [];
  if (e.metaKey) parts.push(isMac ? "Command" : "Super");
  if (e.ctrlKey) parts.push("Control");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");
  if (parts.length === 0 && !STANDALONE.test(e.code)) return { kind: "needs-modifier" };
  parts.push(e.code);
  return { kind: "done", accelerator: parts.join("+") };
}

const MAC_SYMBOLS: Record<string, string> = {
  commandorcontrol: "⌘",
  cmdorctrl: "⌘",
  command: "⌘",
  cmd: "⌘",
  super: "⌘",
  control: "⌃",
  ctrl: "⌃",
  alt: "⌥",
  option: "⌥",
  shift: "⇧",
};

const OTHER_NAMES: Record<string, string> = {
  commandorcontrol: "Ctrl",
  cmdorctrl: "Ctrl",
  command: "Win",
  cmd: "Win",
  super: "Win",
  control: "Ctrl",
  ctrl: "Ctrl",
  alt: "Alt",
  option: "Alt",
  shift: "Shift",
};

const KEY_NAMES: Record<string, string> = {
  arrowup: "↑",
  arrowdown: "↓",
  arrowleft: "←",
  arrowright: "→",
  space: "Space",
  minus: "-",
  equal: "=",
  comma: ",",
  period: ".",
  slash: "/",
  backslash: "\\",
  semicolon: ";",
  quote: "'",
  bracketleft: "[",
  bracketright: "]",
  backquote: "`",
  printscreen: "Print Screen",
  enter: "Enter",
  tab: "Tab",
  backspace: "⌫",
  delete: "Del",
};

/** Human labels for each key of an accelerator, e.g. `["⌘", "⇧", "9"]`. */
export function acceleratorLabels(accelerator: string): string[] {
  return accelerator
    .split("+")
    .filter(Boolean)
    .map((part) => {
      const lower = part.toLowerCase();
      const modifier = (isMac ? MAC_SYMBOLS : OTHER_NAMES)[lower];
      if (modifier) return modifier;
      if (KEY_NAMES[lower]) return KEY_NAMES[lower];
      return part.replace(/^(Key|Digit)/, "").replace(/^Numpad/, "Num ");
    });
}
