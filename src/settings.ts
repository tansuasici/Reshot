// The Settings window. Every change is applied and saved immediately.

import * as ipc from "./ipc";
import { acceleratorFromEvent, acceleratorLabels } from "./accelerator";

const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;

const shortcutButton = $<HTMLButtonElement>("shortcut");
const shortcutError = $<HTMLParagraphElement>("shortcut-error");
const saveDir = $<HTMLElement>("save-dir");
const chooseDir = $<HTMLButtonElement>("choose-dir");
const autostart = $<HTMLInputElement>("autostart");
const autostartError = $<HTMLParagraphElement>("autostart-error");
const formatInputs = document.querySelectorAll<HTMLInputElement>('input[name="format"]');

let current: ipc.Settings;
let recording = false;

function showError(el: HTMLElement, message: string | null) {
  el.textContent = message ?? "";
  el.hidden = message === null;
}

function renderShortcut(accelerator: string) {
  shortcutButton.replaceChildren(
    ...acceleratorLabels(accelerator).map((label) => {
      const kbd = document.createElement("kbd");
      kbd.textContent = label;
      return kbd;
    }),
  );
  shortcutButton.setAttribute("aria-label", `Capture shortcut: ${acceleratorLabels(accelerator).join(" ")}`);
}

function renderSaveDir(dir: string | null) {
  saveDir.textContent = dir ?? "Not set";
  saveDir.title = dir ?? "";
}

// ---- shortcut recorder ----

async function startRecording() {
  if (recording) return;
  recording = true;
  showError(shortcutError, null);
  shortcutButton.classList.add("recording");
  shortcutButton.textContent = "Press keys…";
  // Otherwise pressing the current hotkey here would take a screenshot.
  await ipc.suspendShortcut().catch(() => {});
}

async function stopRecording(accelerator: string | null) {
  if (!recording) return;
  recording = false;
  shortcutButton.classList.remove("recording");
  showError(shortcutError, null);
  if (accelerator && accelerator !== current.shortcut) {
    try {
      await ipc.setShortcut(accelerator);
      current.shortcut = accelerator;
    } catch (e) {
      showError(shortcutError, `${String(e)}. The previous shortcut still works.`);
    }
  }
  await ipc.resumeShortcut().catch((e) => showError(shortcutError, String(e)));
  renderShortcut(current.shortcut);
}

shortcutButton.addEventListener("click", () => void startRecording());
shortcutButton.addEventListener("blur", () => void stopRecording(null));
shortcutButton.addEventListener("keydown", (e) => {
  if (!recording) {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      void startRecording();
    }
    return;
  }
  e.preventDefault();
  e.stopPropagation();
  const noModifiers = !e.metaKey && !e.ctrlKey && !e.altKey && !e.shiftKey;
  if (e.key === "Escape" && noModifiers) {
    void stopRecording(null);
    return;
  }
  const result = acceleratorFromEvent(e);
  if (result.kind === "needs-modifier") {
    showError(shortcutError, "Add ⌘/Ctrl, ⌥/Alt or ⇧ Shift to the key.");
  } else if (result.kind === "done") {
    void stopRecording(result.accelerator);
  }
});

// ---- save folder, format, open at login ----

chooseDir.addEventListener("click", async () => {
  const dir = await ipc.chooseSaveDir().catch(() => null);
  if (dir) renderSaveDir(dir);
});

for (const input of formatInputs) {
  input.addEventListener("change", () => {
    if (input.checked) void ipc.setFormat(input.value as ipc.ImageFormat);
  });
}

autostart.addEventListener("change", async () => {
  const wanted = autostart.checked;
  autostart.disabled = true;
  try {
    autostart.checked = await ipc.setAutostart(wanted);
    showError(autostartError, null);
  } catch (e) {
    autostart.checked = !wanted;
    showError(autostartError, String(e));
  } finally {
    autostart.disabled = false;
  }
});

// ---- init ----

async function init() {
  current = await ipc.getSettings();
  renderShortcut(current.shortcut);
  renderSaveDir(current.saveDir);
  for (const input of formatInputs) input.checked = input.value === current.format;
  autostart.checked = current.autostart;
}

void init();
