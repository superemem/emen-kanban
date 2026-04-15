import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export interface UpdateCheckResult {
  available: boolean;
  version?: string;
  notes?: string;
  message?: string;
}

export async function checkForUpdate(silent = true): Promise<UpdateCheckResult> {
  try {
    const update = await check();
    if (update) {
      return {
        available: true,
        version: update.version,
        notes: update.body,
      };
    }
    return { available: false, message: silent ? undefined : "Udah versi terbaru bro 👌" };
  } catch (e) {
    return { available: false, message: `Gagal cek update: ${e}` };
  }
}

export async function downloadAndInstall(): Promise<void> {
  const update = await check();
  if (!update) throw new Error("Gak ada update tersedia");

  let downloaded = 0;
  let totalBytes = 0;

  await update.downloadAndInstall((event) => {
    switch (event.event) {
      case "Started":
        totalBytes = event.data.contentLength ?? 0;
        console.log(`Download start: ${totalBytes} bytes`);
        break;
      case "Progress":
        downloaded += event.data.chunkLength;
        console.log(`Downloaded ${downloaded}/${totalBytes}`);
        break;
      case "Finished":
        console.log("Download finished, installing...");
        break;
    }
  });

  await relaunch();
}
