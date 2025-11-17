import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  isRegistered,
  register,
  unregister,
} from "@tauri-apps/plugin-global-shortcut";
import useAppStateStore from "@/stores";
import { toast } from "sonner";
let recordStartTime: number | null = null;

export async function toggleRecording() {
  const appState = useAppStateStore.getState();

  if (!appState.isRecording) {
    appState.updateIsRecording(true);
    recordStartTime = Date.now();
  } else {
    if (recordStartTime && Date.now() - recordStartTime < 3000) {
      toast.warning("录音开始后需要等待 3 秒才能停止");
    } else {
      appState.updateIsRecording(false);
      recordStartTime = null;
    }
  }
}


export async function registryGlobalShortCuts() {
  const combo = "CommandOrControl+Shift+`";
  if (await isRegistered(combo)) {
    await unregister(combo);
  }

  await register(combo, async (event) => {
    if (event.state === "Released") {
      const window = getCurrentWindow();

      if (await window.isVisible()) {
        await window.hide();
      } else {
        await window.show();
      }
    }
  });

  const recordCombo = "Alt+Space";
  if (await isRegistered(recordCombo)) {
    await unregister(recordCombo);
  }

  await register(recordCombo, async (event) => {
    if (event.state === "Released") {
      await toggleRecording();
    }
  });
}
