import { Mic, SendHorizontal, Trash2 } from "lucide-react";
import type React from "react";
import {
	useCallback,
	useEffect,
	useEffectEvent,
	useRef,
	useState,
} from "react";
import { toast } from "sonner";
import type { Message } from "@/components/ChatContainer.tsx";
import { MoreMenu } from "@/components/MoreMenu.tsx";
import { Textarea } from "@/components/ui/textarea.tsx";
import Waruls from "@/components/Waruls.tsx";
import { startAudioRecognition, stopAudioRecognition } from "@/lib/audio.ts";
import useAppStateStore from "@/stores";

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
	const [recordingTimerStamp, setRecordingTimerStamp] = useState(() =>
		Date.now(),
	);
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
	const wasRecordingRef = useRef(recordingState);
	const MIN_RECORDING_DURATION = 3000;
	const canStopRecording =
		!recordingState ||
		recordingStartedAt === null ||
		recordingTimerStamp - recordingStartedAt >= MIN_RECORDING_DURATION;

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
		if (!recordingState || recordingStartedAt === null) {
			return () => undefined;
		}

		const remainingDuration = Math.max(
			0,
			recordingStartedAt + MIN_RECORDING_DURATION - Date.now(),
		);
		const timer = setTimeout(() => {
			setRecordingTimerStamp(Date.now());
		}, remainingDuration);

		return () => {
			clearTimeout(timer);
		};
	}, [recordingStartedAt, recordingState]);

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

	const startRecordingEffect = useEffectEvent(() => {
		void startAudioRecognition(
			setInputText,
			currentAudioChannel,
			remoteModelVendor,
			captureInterval,
			isUsePreRecorded,
		);
	});

	const stopRecordingEffect = useEffectEvent(() => {
		void stopAudioRecognition(currentAudioChannel);
	});

	useEffect(() => {
		setIsTyping(recordingState);

		if (recordingState) {
			startRecordingEffect();
		} else if (wasRecordingRef.current) {
			stopRecordingEffect();
		}

		wasRecordingRef.current = recordingState;
	}, [recordingState, setIsTyping]);

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

				{recordingState ? (
					<div className="relative h-16 w-16 shrink-0 overflow-visible">
						<Waruls
							className="h-full w-full"
							onToggle={toggleRecording}
							title={canStopRecording ? "停止语音" : "录音中...3秒后可停止"}
						/>
					</div>
				) : (
					<span title="开始语音">
						<Mic
							onClick={toggleRecording}
							className="cursor-pointer text-gray-400"
						/>
					</span>
				)}

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
					<MoreMenu />
				</span>
			</div>
		</div>
	);
};
