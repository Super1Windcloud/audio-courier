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
import { setRecordingStateImmediately } from "@/lib/recordingState.ts";
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
	const [finalTranscript, setFinalTranscript] = useState("");
	const [recordingTimerStamp, setRecordingTimerStamp] = useState(() =>
		Date.now(),
	);
	const recordingState = useAppStateStore((state) => state.isRecording);
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
	const shouldUseWaruls =
		typeof navigator !== "undefined" && navigator.userAgent.includes("Windows");
	const normalizeTranscriptForDuplicateGuard = useCallback(
		(text: string) => {
			if (remoteModelVendor !== "assemblyai") {
				return text;
			}

			return text
				.replace(/\s+/g, "")
				.replace(/[。．.，,、！!？?；;：:“”"'‘’（）()[\]【】]+$/g, "");
		},
		[remoteModelVendor],
	);

	const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
	const textareaRef = useRef<HTMLTextAreaElement | null>(null);
	const wasRecordingRef = useRef(recordingState);
	const lastSubmittedRef = useRef<{ text: string; at: number } | null>(null);
	const MIN_RECORDING_DURATION = 3000;
	const DUPLICATE_SEND_GUARD_MS = 2000;
	const canStopRecording =
		!recordingState ||
		recordingStartedAt === null ||
		recordingTimerStamp - recordingStartedAt >= MIN_RECORDING_DURATION;
	const recordingTitle = canStopRecording ? "停止语音" : "录音中...3秒后可停止";

	const handleSend = useCallback(
		(overrideText?: string) => {
			if (!isAuthorized) {
				toast.warning("未激活许可证，无法使用发送和录音功能");
				return;
			}

			const text = (overrideText ?? inputText).trim();
			if (text) {
				const now = Date.now();
				const lastSubmitted = lastSubmittedRef.current;
				const normalizedText = normalizeTranscriptForDuplicateGuard(text);
				if (
					lastSubmitted &&
					lastSubmitted.text === normalizedText &&
					now - lastSubmitted.at < DUPLICATE_SEND_GUARD_MS
				) {
					return;
				}

				lastSubmittedRef.current = { text: normalizedText, at: now };
				updateQuestionState(text);
				onSendMessage(text);
				setInputText((current) => (current === text ? "" : current));
				setFinalTranscript((current) => (current === text ? "" : current));
			}
		},
		[
			inputText,
			isAuthorized,
			normalizeTranscriptForDuplicateGuard,
			onSendMessage,
			updateQuestionState,
		],
	);

	useEffect(() => {
		if (!finalTranscript.trim()) return;

		if (timeoutRef.current) {
			clearTimeout(timeoutRef.current);
		}

		let timeout: number;
		if (remoteModelVendor === "assemblyai") {
			timeout = 0;
		} else if (remoteModelVendor === "gladia") {
			timeout = 100;
		} else if (
			remoteModelVendor === "deepgram" ||
			remoteModelVendor === "revai"
		) {
			timeout = 200;
		} else {
			timeout = 1000;
		}
		timeoutRef.current = setTimeout(() => {
			handleSend(finalTranscript);
		}, timeout);

		return () => {
			if (timeoutRef.current) clearTimeout(timeoutRef.current);
		};
	}, [finalTranscript, handleSend, remoteModelVendor]);

	const handleKeyPress = (e: React.KeyboardEvent) => {
		if (e.key === "Enter" && !e.shiftKey) {
			e.preventDefault();
			handleSend();
		}
	};

	const handleGlobalSendShortcut = useEffectEvent((event: KeyboardEvent) => {
		if (event.key !== "Enter" || !event.shiftKey || event.isComposing) {
			return;
		}

		const activeElement = document.activeElement;
		if (activeElement === textareaRef.current) {
			return;
		}

		if (
			activeElement instanceof HTMLInputElement ||
			activeElement instanceof HTMLTextAreaElement ||
			activeElement instanceof HTMLSelectElement ||
			activeElement?.getAttribute("contenteditable") === "true"
		) {
			return;
		}

		event.preventDefault();
		handleSend();
	});

	useEffect(() => {
		const onKeyDown = (event: KeyboardEvent) => {
			handleGlobalSendShortcut(event);
		};

		window.addEventListener("keydown", onKeyDown);
		return () => {
			window.removeEventListener("keydown", onKeyDown);
		};
	}, []);

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

	const toggleRecording = () => {
		if (!isAuthorized) {
			toast.warning("未激活许可证，无法开始录音");
			return;
		}

		if (!recordingState) {
			setRecordingStateImmediately(true);
			return;
		}

		const startedAt = recordingStartedAt ?? 0;
		const elapsed = Date.now() - startedAt;
		if (elapsed < MIN_RECORDING_DURATION) {
			toast.warning("录音开始后需要等待3秒才能停止");
			return;
		}
		setRecordingStateImmediately(false);
	};

	const startRecordingEffect = useEffectEvent(() => {
		void startAudioRecognition(
			setInputText,
			(message) => {
				setInputText(message);
				setFinalTranscript(message);
			},
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
		setFinalTranscript("");

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
					ref={textareaRef}
					value={inputText}
					onChange={(e) => {
						setInputText(e.target.value);
						setFinalTranscript("");
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
					shouldUseWaruls ? (
						<div
							key="recording-waruls"
							className="relative h-8 w-8 shrink-0 overflow-visible"
						>
							<Waruls
								className="h-full w-full"
								onToggle={toggleRecording}
								scale={0.32}
								title={recordingTitle}
							/>
						</div>
					) : (
						<span key="recording-mic" title={recordingTitle}>
							<Mic
								onClick={toggleRecording}
								className="cursor-pointer text-red-500"
							/>
						</span>
					)
				) : (
					<span key="idle-mic" title="开始语音">
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
						onClick={() => handleSend()}
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
