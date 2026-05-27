import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";

let permissionGranted: boolean | null = null;

async function ensureNotificationPermission(): Promise<boolean> {
  if (permissionGranted === true) return true;

  try {
    if (await isPermissionGranted()) {
      permissionGranted = true;
      return true;
    }

    permissionGranted = (await requestPermission()) === "granted";
    return permissionGranted;
  } catch (e) {
    console.debug("[vibemux] notification permission failed:", e);
    permissionGranted = false;
    return false;
  }
}

export async function sendSessionNotification(title: string, body: string): Promise<void> {
  if (!(await ensureNotificationPermission())) return;

  try {
    sendNotification({ title, body });
  } catch (e) {
    console.debug("[vibemux] send notification failed:", e);
  }
}
