import { Mic, SendHorizontal, Trash2 } from "lucide-react";
import type React from "react";
import { lazy, Suspense, useCallback, useEffect, useRef, useState } from "react";
import { toast } from "sonner";
import type { Message } from "@/components/ChatContainer.tsx";
import { Textarea } from "@/components/ui/textarea.tsx";
import { startAudioRecognition, stopAudioRecognition } from "@/lib/audio.ts";
import useAppStateStore from "@/stores";

const MoreMenu = lazy(() =>
	import("@/components/MoreMenu.tsx").then((module) => ({
		default: module.MoreMenu,
	})),
);

interface MessageInputProps {
	onSendMessage: (text: string) => void;
	onClearConversation: () => void;
	setMessages: React.Dispatch<React.SetStateAction<Message[]>>;
	setIsTyping: (record: boolean) => void;
}

export const MessageInput: React.FC<MessageInputProps> = ({
	onSendMessage,
	onClearConversation,
	setIsTyping,
}) => {
	const [inputText, setInputText] = useState("");
	const [canStopRecording, setCanStopRecording] = useState(true);
	const recordingState = useAppStateStore((state) => state.isRecording);
	const updateRecordingState = useAppStateStore(
		(state) => state.updateIsRecording,
	);
	const captureInterval = useAppStateStore((state) => state.captureInterval);
	const updateQuestionState = useAppStateStore((state) => state.updateQuestion);
	const remoteModelVendor = useAppStateStore(
		(state) => state.useRemoteModelTranscribe,
	);
	const currentAudioChannel = useAppStateStore(
		(state) => state.currentAudioChannel,
	);
	const isUsePreRecorded = useAppStateStore((state) => state.isUsePreRecorded);
	const recordingStartedAt = useAppStateStore(
		(state) => state.recordingStartedAt,
	);
	const licenseStatus = useAppStateStore((state) => state.licenseStatus);
	const isAuthorized = Boolean(
		licenseStatus?.isValid || licenseStatus?.isHostSigner,
	);

	const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
	const MIN_RECORDING_DURATION = 3000;

	const handleSend = useCallback(() => {
		if (!isAuthorized) {
			toast.warning("未激活许可证，无法使用发送和录音功能");
			return;
		}

		if (inputText.trim()) {
			updateQuestionState(inputText.trim());
			onSendMessage(inputText.trim());
			setInputText("");
		}
	}, [inputText, isAuthorized, onSendMessage, updateQuestionState]);

	useEffect(() => {
		if (!recordingState) return () => undefined;
		if (!inputText.trim()) return;

		if (timeoutRef.current) {
			clearTimeout(timeoutRef.current);
		}

		let timeout: number;
		if (remoteModelVendor === "assemblyai" || remoteModelVendor === "gladia") {
			timeout = 100;
		} else if (remoteModelVendor === "deepgram") {
			timeout = 200;
		} else {
			timeout = 1000;
		}
		timeoutRef.current = setTimeout(() => {
			handleSend();
		}, timeout);

		return () => {
			if (timeoutRef.current) clearTimeout(timeoutRef.current);
		};
	}, [handleSend, inputText, recordingState, remoteModelVendor]);

	const handleKeyPress = (e: React.KeyboardEvent) => {
		if (e.key === "Enter" && !e.shiftKey) {
			e.preventDefault();
			handleSend();
		}
	};

	useEffect(() => {
		if (!recordingState) {
			setCanStopRecording(true);
			return;
		}

		setCanStopRecording(false);
		const timer = setTimeout(() => {
			setCanStopRecording(true);
		}, MIN_RECORDING_DURATION);

		return () => {
			clearTimeout(timer);
		};
	}, [recordingState]);

	const toggleRecording = async () => {
		if (!isAuthorized) {
			toast.warning("未激活许可证，无法开始录音");
			return;
		}

		if (!recordingState) {
			updateRecordingState(true);
			return;
		}

		const startedAt = recordingStartedAt ?? 0;
		const elapsed = Date.now() - startedAt;
		if (elapsed < MIN_RECORDING_DURATION) {
			toast.warning("录音开始后需要等待3秒才能停止");
			return;
		}
		updateRecordingState(false);
	};

	useEffect(() => {
		setIsTyping(recordingState);
		if (recordingState) {
			void startAudioRecognition(
				setInputText,
				currentAudioChannel,
				remoteModelVendor,
				captureInterval,
				isUsePreRecorded,
			);
			return;
		}

		void stopAudioRecognition(currentAudioChannel);
	}, [
		captureInterval,
		currentAudioChannel,
		isUsePreRecorded,
		recordingState,
		remoteModelVendor,
		setIsTyping,
	]);

	const handleClearConversation = () => {
		setInputText("");

		onClearConversation();
	};

	return (
		<div
			className="p-4
      relative   px-4 py-2 shadow-sm
      backdrop-blur-xl  bg-white/10 border border-white/10"
		>
			<div className="flex border-none items-center space-x-2">
				<Textarea
					value={inputText}
					onChange={(e) => {
						setInputText(e.target.value);
						e.currentTarget.style.height = "auto"; // 先重置
						e.currentTarget.style.height = `${e.currentTarget.scrollHeight}px`; // 根据内容调整
					}}
					onKeyDown={handleKeyPress}
					placeholder="输入消息..."
					rows={1}
					disabled={!isAuthorized}
					className="flex-1 resize-none overflow-hidden text-white border-none focus-visible:ring-0 placeholder:text-gray-300 focus-visible:ring-offset-0 bg-transparent"
				/>

				<span
					title={
						recordingState
							? canStopRecording
								? "停止语音"
								: "录音中...3秒后可停止"
							: "开始语音"
					}
				>
					<Mic
						onClick={toggleRecording}
						className={`cursor-pointer ${recordingState ? (canStopRecording ? "text-red-500" : "text-orange-500") : "text-gray-400"}`}
					/>
				</span>

				<span title="清空会话">
					<Trash2
						onClick={handleClearConversation}
						className="cursor-pointer text-gray-400"
					/>
				</span>

				<span title="发送消息">
					<SendHorizontal
						onClick={handleSend}
						className="text-gray-400 cursor-pointer"
					/>
				</span>

				<span title="更多选项">
					<Suspense fallback={<span className="text-gray-400">...</span>}>
						<MoreMenu />
					</Suspense>
				</span>
			</div>
		</div>
	);
};
