import { flushSync } from "react-dom";
import useAppStateStore from "@/stores";

export function setRecordingStateImmediately(target: boolean) {
	const appState = useAppStateStore.getState();

	if (appState.isRecording === target) {
		return;
	}

	flushSync(() => {
		useAppStateStore.getState().updateIsRecording(target);
	});
}
