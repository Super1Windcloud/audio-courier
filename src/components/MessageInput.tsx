import { Mic, SendHorizontal, Trash2 } from "lucide-react";
import React, { useEffect, useRef, useState } from "react";

import { Message } from "@/components/ChatContainer.tsx";
import { MoreMenu } from "@/components/MoreMenu.tsx";
import { Textarea } from "@/components/ui/textarea.tsx";
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

	const timeoutRef = useRef<NodeJS.Timeout | null>(null);

	const handleSend = () => {
		if (inputText.trim()) {
			updateQuestionState(inputText.trim());
			onSendMessage(inputText.trim());
			setInputText("");
		}
	};

	useEffect(() => {
		if (!recordingState) return () => undefined;
		if (!inputText.trim()) return;

		if (timeoutRef.current) {
			clearTimeout(timeoutRef.current);
		}

		let timeout: number;
		if (remoteModelVendor === "assemblyai") {
			timeout = 100;
		} else {
			timeout = 1000;
		}
		timeoutRef.current = setTimeout(() => {
			handleSend();
		}, timeout);

		return () => {
			if (timeoutRef.current) clearTimeout(timeoutRef.current);
		};
	}, [inputText, recordingState]);

	const handleKeyPress = (e: React.KeyboardEvent) => {
		if (e.key === "Enter" && !e.shiftKey) {
			e.preventDefault();
			handleSend();
		}
	};

	const toggleRecording = async () => {
		if (!recordingState) {
			updateRecordingState(true);
		} else {
			updateRecordingState(false);
		}
	};

	useEffect(() => {
		setIsTyping(recordingState);
		if (recordingState) {
			startAudioRecognition(
				setInputText,
				currentAudioChannel,
				remoteModelVendor,
				captureInterval,
				isUsePreRecorded,
			).then();
		} else {
			stopAudioRecognition(currentAudioChannel).then();
		}
	}, [recordingState]);

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
						e.currentTarget.style.height = e.currentTarget.scrollHeight + "px"; // 根据内容调整
					}}
					onKeyDown={handleKeyPress}
					placeholder="输入消息..."
					rows={1}
					className="flex-1 resize-none overflow-hidden text-white border-none focus-visible:ring-0 placeholder:text-gray-300 focus-visible:ring-offset-0 bg-transparent"
				/>

				<span title={recordingState ? "停止语音" : "开始语音"}>
					<Mic
						onClick={toggleRecording}
						className={`cursor-pointer ${recordingState ? "text-red-500" : "text-gray-400"}`}
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
					<MoreMenu />
				</span>
			</div>
		</div>
	);
};
