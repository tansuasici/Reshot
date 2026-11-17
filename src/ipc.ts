// The ONLY module that talks to the Tauri backend. Outside Tauri (plain
// `npm run dev` in a browser) it serves a generated fixture frame, so the
// overlay UI can be developed and tested without capturing the screen.

export interface FrameMeta {
  /** Captured image size in physical pixels. */
  width: number;
  height: number;
  /** Viewport size the overlay will have once the backend has placed it. */
  cssWidth: number;
  cssHeight: number;
}

export interface Frame {
  width: number;
  height: number;
  pixels: Uint8ClampedArray;
}

export interface RgbaImage {
  width: number;
  height: number;
  pixels: Uint8ClampedArray;
}

const inTauri = "__TAURI_INTERNALS__" in window;

async function tauriInvoke<T>(
  cmd: string,
  payload?: Uint8Array | Record<string, unknown>,
  headers?: Record<string, string>,
): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, payload, headers ? { headers } : undefined);
}

function imageHeaders(img: RgbaImage, extra: Record<string, string> = {}) {
  return { "x-width": String(img.width), "x-height": String(img.height), ...extra };
}

function bytesOf(img: RgbaImage): Uint8Array {
  return new Uint8Array(img.pixels.buffer, img.pixels.byteOffset, img.pixels.byteLength);
}

/** Call `handler` every time the backend has a new capture ready. */
export async function onCapture(handler: (meta: FrameMeta) => void): Promise<void> {
  if (!inTauri) {
    // Browser fixture: behave as if a capture just happened, once the
    // viewport has a real size (a hidden tab can load at 0x0).
    const fire = () => {
      if (window.innerWidth === 0 || window.innerHeight === 0) {
        window.addEventListener("resize", fire, { once: true });
        return;
      }
      handler({
        width: Math.round(window.innerWidth * devicePixelRatio),
        height: Math.round(window.innerHeight * devicePixelRatio),
        cssWidth: window.innerWidth,
        cssHeight: window.innerHeight,
      });
    };
    setTimeout(fire, 0);
    return;
  }
  const { listen } = await import("@tauri-apps/api/event");
  await listen<FrameMeta>("capture-ready", (e) => handler(e.payload));
}

/** Tell the backend the capture listener is registered (triggers the launch capture). */
export async function overlayReady(): Promise<void> {
  if (inTauri) await tauriInvoke("overlay_ready");
}

/** Fetch the captured frame's raw RGBA pixels. */
export async function getFrame(meta: FrameMeta): Promise<Frame> {
  const { width, height } = meta;
  if (!inTauri) return { width, height, pixels: fixturePixels(meta) };
  const buffer = await tauriInvoke<ArrayBuffer>("get_frame");
  return { width, height, pixels: new Uint8ClampedArray(buffer) };
}

/** Show the (already painted) overlay window. */
export async function showOverlay(): Promise<void> {
  if (inTauri) await tauriInvoke("show_overlay");
}

/** Make the shown overlay visible (it is shown transparent until painted). */
export async function revealOverlay(): Promise<void> {
  if (inTauri) await tauriInvoke("reveal_overlay");
}

/** Hide the overlay window. */
export async function closeOverlay(): Promise<void> {
  if (inTauri) await tauriInvoke("close_overlay");
  else console.info("[reshot] close");
}

/** Put an image on the system clipboard. */
export async function copyImage(img: RgbaImage): Promise<void> {
  if (!inTauri) {
    console.info(`[reshot] copy ${img.width}x${img.height}`);
    return;
  }
  await tauriInvoke("copy_image", bytesOf(img), imageHeaders(img));
}

/** Ask where to save and write the file. Resolves `false` if cancelled. */
export async function saveImage(img: RgbaImage, fileName: string): Promise<boolean> {
  if (!inTauri) {
    console.info(`[reshot] save ${img.width}x${img.height} as ${fileName}`);
    return true;
  }
  return tauriInvoke<boolean>("save_image", bytesOf(img), imageHeaders(img, { "x-filename": fileName }));
}

// ---- Settings window ----

export type ImageFormat = "png" | "jpeg";

export interface Settings {
  /** Capture hotkey as a Tauri accelerator, e.g. `CommandOrControl+Shift+9`. */
  shortcut: string;
  /** Folder the save dialog opens in. */
  saveDir: string | null;
  format: ImageFormat;
  /** Open at login, as the OS reports it. */
  autostart: boolean;
}

const fixtureSettings: Settings = {
  shortcut: "CommandOrControl+Shift+9",
  saveDir: "/Users/you/Pictures",
  format: "png",
  autostart: false,
};

export async function getSettings(): Promise<Settings> {
  if (!inTauri) return { ...fixtureSettings };
  return tauriInvoke<Settings>("get_settings");
}

/** Switch the capture hotkey. Rejects (old one kept) if it can't be registered. */
export async function setShortcut(accelerator: string): Promise<void> {
  if (!inTauri) {
    fixtureSettings.shortcut = accelerator;
    return;
  }
  await tauriInvoke("set_shortcut", { accelerator });
}

/** Release the hotkey while a new one is being recorded. */
export async function suspendShortcut(): Promise<void> {
  if (inTauri) await tauriInvoke("suspend_shortcut");
}

/** Re-register the saved hotkey. */
export async function resumeShortcut(): Promise<void> {
  if (inTauri) await tauriInvoke("resume_shortcut");
}

/** Pick the save folder. Resolves the new folder, or `null` if cancelled. */
export async function chooseSaveDir(): Promise<string | null> {
  if (!inTauri) return fixtureSettings.saveDir;
  return tauriInvoke<string | null>("choose_save_dir");
}

export async function setFormat(format: ImageFormat): Promise<void> {
  if (inTauri) await tauriInvoke("set_format", { format });
  else fixtureSettings.format = format;
}

/** Turn open-at-login on/off. Resolves the state the OS reports afterwards. */
export async function setAutostart(enabled: boolean): Promise<boolean> {
  if (!inTauri) return enabled;
  return tauriInvoke<boolean>("set_autostart", { enabled });
}

function fixturePixels({ width, height }: FrameMeta): Uint8ClampedArray {
  const px = new Uint8ClampedArray(width * height * 4);
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const i = (y * width + x) * 4;
      const grid = (x >> 6) % 2 === (y >> 6) % 2;
      px[i] = (x / width) * 200 + (grid ? 40 : 0);
      px[i + 1] = (y / height) * 180 + (grid ? 40 : 0);
      px[i + 2] = 160;
      px[i + 3] = 255;
    }
  }
  return px;
}
