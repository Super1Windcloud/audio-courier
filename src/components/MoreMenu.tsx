import { invoke } from "@tauri-apps/api/core";
import { MoreVertical, RotateCcw, Save, Sparkles } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";
import { LlmProviderDialog } from "@/components/LlmProviderDialog.tsx";
import { TranscriptProviderDialog } from "@/components/TranscriptProviderDialog.tsx";
import { Button } from "@/components/ui/button.tsx";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog.tsx";
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSub,
	DropdownMenuSubContent,
	DropdownMenuSubTrigger,
	DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu.tsx";
import { Textarea } from "@/components/ui/textarea.tsx";
import { runUpdater } from "@/lib/updater.ts";
import useAppStateStore, {
	type TranscribeVendor,
	type UiTextTone,
} from "@/stores";
import { HOTKEYS, MODEL_LABELS, MODEL_OPTIONS } from "@/types/llm.ts";
import { TRANSCRIBE_VENDOR_LABELS } from "@/types/provider.ts";

export function MoreMenu() {
	const appState = useAppStateStore();
	const currentAudioChannel = useAppStateStore(
		(state) => state.currentAudioChannel,
	);
	const currentModel = useAppStateStore((state) => state.currentSelectedModel);
	const updateCurrentAudioChannel = useAppStateStore(
		(state) => state.updateCurrentAudioChannel,
	);
	const updateCurrentSelectedModel = useAppStateStore(
		(state) => state.updateCurrentSelectedModel,
	);
	const [audioChannels, setAudioChannels] = useState<string[]>([]);
	const [isPromptDialogOpen, setIsPromptDialogOpen] = useState(false);
	const [isLlmConfigDialogOpen, setIsLlmConfigDialogOpen] = useState(false);
	const [isTranscriptConfigDialogOpen, setIsTranscriptConfigDialogOpen] =
		useState(false);
	const [isUpdating, setIsUpdating] = useState(false);
	const [promptDraft, setPromptDraft] = useState(appState.llmPrompt);
	const [interviewPromptDraft, setInterviewPromptDraft] = useState(
		appState.interviewPrompt,
	);
	const TRANSCRIBE_VENDORS: TranscribeVendor[] = [
		"assemblyai",
		"deepgram",
		"gladia",
		"revai",
		"speechmatics",
	];
	const UI_OPACITY_OPTIONS = [
		100, 95, 90, 85, 80, 75, 70, 65, 60, 55, 50, 45, 40, 35, 30,
	];
	const UI_TEXT_TONE_OPTIONS: { value: UiTextTone; label: string }[] = [
		{ value: "light", label: "浅色文字" },
		{ value: "dark", label: "深色文字" },
	];
	const defaultPrompt = import.meta.env.VITE_PROMPT || "";
	const defaultInterviewPrompt = import.meta.env.VITE_INTERVIEW_PROMPT || "";
	const modelLabels = {
		...MODEL_LABELS,
		custom_openai:
			appState.llmProviderSettings.customOpenAiName.trim() ||
			MODEL_LABELS.custom_openai,
	};
	const shouldOpenPromptDialogOnStartup =
		defaultPrompt.trim().length === 0 ||
		defaultInterviewPrompt.trim().length === 0;
	const hasPromptChanges =
		promptDraft !== appState.llmPrompt ||
		interviewPromptDraft !== appState.interviewPrompt;
	const isUsingDefaultPrompt =
		promptDraft === defaultPrompt &&
		interviewPromptDraft === defaultInterviewPrompt;

	useEffect(() => {
		void invoke<string[]>("get_audio_stream_devices_names")
			.then((result) => {
				setAudioChannels(result);
				if (result.length === 0) {
					toast.error("No audio streams output device found");
					return;
				}

				if (!currentAudioChannel || !result.includes(currentAudioChannel)) {
					updateCurrentAudioChannel(result[0]);
				}
			})
			.catch((error) => {
				toast.error(String(error));
			});
	}, [currentAudioChannel, updateCurrentAudioChannel]);

	useEffect(() => {
		if (!isPromptDialogOpen) {
			return;
		}

		setPromptDraft(appState.llmPrompt);
		setInterviewPromptDraft(appState.interviewPrompt);
	}, [appState.interviewPrompt, appState.llmPrompt, isPromptDialogOpen]);

	useEffect(() => {
		if (!shouldOpenPromptDialogOnStartup) {
			return;
		}

		setIsPromptDialogOpen(true);
	}, [shouldOpenPromptDialogOnStartup]);

	const handleCheckUpdate = async () => {
		if (isUpdating) {
			return;
		}

		setIsUpdating(true);
		try {
			await runUpdater();
		} finally {
			setIsUpdating(false);
		}
	};

	const handlePromptSave = () => {
		appState.updateLLMPrompt(promptDraft);
		appState.updateInterviewPrompt(interviewPromptDraft);
		setIsPromptDialogOpen(false);
		toast.success("提示词已保存");
	};

	const handlePromptReset = () => {
		setPromptDraft(defaultPrompt);
		setInterviewPromptDraft(defaultInterviewPrompt);
	};

	return (
		<>
			<DropdownMenu>
				<DropdownMenuTrigger asChild>
					<MoreVertical className="text-gray-400 cursor-pointer bg-transparent" />
				</DropdownMenuTrigger>
				<DropdownMenuContent
					align="end"
					className="w-40 bg-gray-600 text-white border-0"
				>
					<DropdownMenuItem
						onClick={() => setIsPromptDialogOpen(true)}
						className="data-[highlighted]:bg-gray-500"
					>
						提示词
					</DropdownMenuItem>
					<DropdownMenuItem
						onClick={() => setIsLlmConfigDialogOpen(true)}
						className="data-[highlighted]:bg-gray-500"
					>
						模型 API
					</DropdownMenuItem>
					<DropdownMenuItem
						onClick={() => setIsTranscriptConfigDialogOpen(true)}
						className="data-[highlighted]:bg-gray-500"
					>
						转录 API
					</DropdownMenuItem>
					<DropdownMenuItem
						onClick={handleCheckUpdate}
						disabled={isUpdating}
						className="data-[highlighted]:bg-gray-500"
					>
						{isUpdating ? "检查更新中..." : "检查更新"}
					</DropdownMenuItem>
					<DropdownMenuSub>
						<DropdownMenuSubTrigger
							className="
             bg-gray-600 text-white
            data-[highlighted]:bg-gray-500
            data-[state=open]:bg-gray-500"
						>
							前端透明度
						</DropdownMenuSubTrigger>
						<DropdownMenuSubContent className="w-40 bg-gray-600 text-white border-0">
							{UI_OPACITY_OPTIONS.map((opacity) => {
								const normalizedOpacity = opacity / 100;
								return (
									<DropdownMenuItem
										key={opacity}
										className={`data-[highlighted]:bg-gray-500 ${
											appState.uiOpacity === normalizedOpacity
												? "font-bold"
												: ""
										}`}
										onClick={() => appState.updateUiOpacity(normalizedOpacity)}
									>
										{opacity}%
										{appState.uiOpacity === normalizedOpacity && (
											<span className="ml-2 text-green-400">✔</span>
										)}
									</DropdownMenuItem>
								);
							})}
						</DropdownMenuSubContent>
					</DropdownMenuSub>
					<DropdownMenuSub>
						<DropdownMenuSubTrigger
							className="
             bg-gray-600 text-white
            data-[highlighted]:bg-gray-500
            data-[state=open]:bg-gray-500"
						>
							文字颜色
						</DropdownMenuSubTrigger>
						<DropdownMenuSubContent className="w-40 bg-gray-600 text-white border-0">
							{UI_TEXT_TONE_OPTIONS.map((option) => (
								<DropdownMenuItem
									key={option.value}
									className={`data-[highlighted]:bg-gray-500 ${
										appState.uiTextTone === option.value ? "font-bold" : ""
									}`}
									onClick={() => appState.updateUiTextTone(option.value)}
								>
									{option.label}
									{appState.uiTextTone === option.value && (
										<span className="ml-2 text-green-400">✔</span>
									)}
								</DropdownMenuItem>
							))}
						</DropdownMenuSubContent>
					</DropdownMenuSub>
					<DropdownMenuSub>
						<DropdownMenuSubTrigger
							className="
             bg-gray-600 text-white
            data-[highlighted]:bg-gray-500
            data-[state=open]:bg-gray-500"
						>
							快捷键
						</DropdownMenuSubTrigger>
						<DropdownMenuSubContent className="w-48 bg-gray-600 text-white border-0">
							{HOTKEYS.map((key) => (
								<DropdownMenuItem
									key={key}
									className={`data-[highlighted]:bg-gray-500`}
								>
									{key}
								</DropdownMenuItem>
							))}
						</DropdownMenuSubContent>
					</DropdownMenuSub>
					<DropdownMenuSub>
						<DropdownMenuSubTrigger
							className="
             bg-gray-600 text-white
            data-[highlighted]:bg-gray-500
            data-[state=open]:bg-gray-500"
						>
							大模型
						</DropdownMenuSubTrigger>
						<DropdownMenuSubContent className="w-48 bg-gray-600 text-white border-0">
							{MODEL_OPTIONS.map((model) => (
								<DropdownMenuItem
									key={model}
									className={`data-[highlighted]:bg-gray-500 ${
										currentModel === model ? "font-bold" : ""
									}`}
									onClick={() => updateCurrentSelectedModel(model)}
								>
									{modelLabels[model]}
									{currentModel === model && (
										<span className="ml-2 text-green-400">✔</span>
									)}
								</DropdownMenuItem>
							))}
						</DropdownMenuSubContent>
					</DropdownMenuSub>
					<DropdownMenuSub>
						<DropdownMenuSubTrigger
							className="
              bg-gray-600 text-white
             data-[highlighted]:bg-gray-500
             data-[state=open]:bg-gray-500"
						>
							转录厂商
						</DropdownMenuSubTrigger>
						<DropdownMenuSubContent className="w-48 bg-gray-600 text-white border-0">
							{TRANSCRIBE_VENDORS.map((vendor) => (
								<DropdownMenuItem
									key={vendor}
									className={`data-[highlighted]:bg-gray-500 ${
										appState.useRemoteModelTranscribe === vendor
											? "font-bold"
											: ""
									}`}
									onClick={() => appState.updateRemoteModelTranscribe(vendor)}
								>
									{TRANSCRIBE_VENDOR_LABELS[vendor]}
									{appState.useRemoteModelTranscribe === vendor && (
										<span className="ml-2 text-green-400">✔</span>
									)}
								</DropdownMenuItem>
							))}
						</DropdownMenuSubContent>
					</DropdownMenuSub>
					<DropdownMenuSub>
						<DropdownMenuSubTrigger
							className="
             bg-gray-600 text-white
            data-[highlighted]:bg-gray-500
            data-[state=open]:bg-gray-500"
						>
							选择音频通道
						</DropdownMenuSubTrigger>
						<DropdownMenuSubContent className="w-48 bg-gray-600 text-white border-0">
							{audioChannels.map((devices) => (
								<DropdownMenuItem
									key={devices}
									className={`data-[highlighted]:bg-gray-500 ${
										appState.currentAudioChannel === devices ? "font-bold" : ""
									}`}
									onClick={() => appState.updateCurrentAudioChannel(devices)}
								>
									{devices}
									{appState.currentAudioChannel === devices && (
										<span className="ml-2 text-green-400">✔</span>
									)}
								</DropdownMenuItem>
							))}
						</DropdownMenuSubContent>
					</DropdownMenuSub>
					<DropdownMenuItem
						onClick={() => {
							appState.updateScrollToBottom(!appState.isStartScrollToBottom);
						}}
						className="flex items-center bg-gray-600 !hover:bg-gray-600  justify-between"
					>
						<span>自动滚动到底部</span>
						<input
							type="checkbox"
							checked={appState.isStartScrollToBottom}
							onChange={(e) => appState.updateScrollToBottom(e.target.checked)}
						/>
					</DropdownMenuItem>
					{/*<DropdownMenuItem*/}
					{/*	onClick={() => {*/}
					{/*		appState.updatePreRecorded(!appState.isUsePreRecorded);*/}
					{/*	}}*/}
					{/*	className="flex items-center bg-gray-600 !hover:bg-gray-600  justify-between"*/}
					{/*>*/}
					{/*	<span>是否使用预录制</span>*/}
					{/*	<input*/}
					{/*		type="checkbox"*/}
					{/*		checked={appState.isUsePreRecorded}*/}
					{/*		onChange={(e) => appState.updatePreRecorded(e.target.checked)}*/}
					{/*	/>*/}
					{/*</DropdownMenuItem>*/}
					<DropdownMenuSub>
						<DropdownMenuSubTrigger
							className="
             bg-gray-600 text-white
            data-[highlighted]:bg-gray-500
            data-[state=open]:bg-gray-500"
						>
							捕获间隔
						</DropdownMenuSubTrigger>
						<DropdownMenuSubContent className="w-48 bg-gray-600 text-white border-0">
							{[1, 2, 3, 5, 10].map((interval) => (
								<DropdownMenuItem
									key={interval}
									className={`data-[highlighted]:bg-gray-500 ${
										appState.captureInterval === interval ? "font-bold" : ""
									}`}
									onClick={() => appState.updateCaptureInterval(interval)}
								>
									{interval}
									{appState.captureInterval === interval && (
										<span className="ml-2 text-green-400">✔</span>
									)}
								</DropdownMenuItem>
							))}
						</DropdownMenuSubContent>
					</DropdownMenuSub>
				</DropdownMenuContent>
			</DropdownMenu>
			<LlmProviderDialog
				open={isLlmConfigDialogOpen}
				onOpenChange={setIsLlmConfigDialogOpen}
			/>
			<TranscriptProviderDialog
				open={isTranscriptConfigDialogOpen}
				onOpenChange={setIsTranscriptConfigDialogOpen}
			/>
			<Dialog open={isPromptDialogOpen} onOpenChange={setIsPromptDialogOpen}>
				<DialogContent className="prompt-editor-dialog max-h-[88vh] overflow-hidden border-none p-0 text-white sm:max-w-[760px]">
					<div className="prompt-editor-shell">
						<div className="prompt-editor-orb prompt-editor-orb-left" />
						<div className="prompt-editor-orb prompt-editor-orb-right" />
						<div className="prompt-editor-orb prompt-editor-orb-bottom" />
						<div className="prompt-editor-content">
							<DialogHeader className="prompt-editor-header">
								<div className="prompt-editor-kicker">
									<Sparkles className="h-3.5 w-3.5" />
									System Prompt
								</div>
								<DialogTitle className="prompt-editor-title">
									同时管理常规对话与自我介绍提示词
								</DialogTitle>
								<DialogDescription className="prompt-editor-description">
									常规提示词会参与后续请求；自我介绍提示词只用于开场自我介绍那一次请求，不会参与后续请求。
								</DialogDescription>
							</DialogHeader>

							<div className="prompt-editor-status-row">
								{shouldOpenPromptDialogOnStartup ? (
									<div className="prompt-editor-chip">
										检测到默认提示词缺失，请先补全并保存
									</div>
								) : null}
								<div className="prompt-editor-chip">
									{hasPromptChanges ? "未保存更改" : "当前已同步"}
								</div>
								<div className="prompt-editor-chip prompt-editor-chip-muted">
									常规 {promptDraft.length} 字符
								</div>
								<div className="prompt-editor-chip prompt-editor-chip-muted">
									自我介绍 {interviewPromptDraft.length} 字符
								</div>
							</div>

							<div className="prompt-editor-scroll">
								<div className="prompt-editor-field">
									<div className="prompt-editor-field-label">
										<span>常规对话提示词</span>
										<span>用于所有普通聊天与语音转文字后的对话</span>
									</div>
									<p className="prompt-editor-field-description">
										建议描述长期角色设定、回答风格、语言偏好和输出深度。
									</p>
									<Textarea
										value={promptDraft}
										onChange={(e) => {
											setPromptDraft(e.target.value);
										}}
										placeholder="请输入常规对话提示词..."
										className="prompt-editor-textarea"
										autoFocus
										style={{
											scrollbarWidth: "none",
										}}
										rows={8}
										onFocus={(e) =>
											e.currentTarget.setSelectionRange(
												e.currentTarget.value.length,
												e.currentTarget.value.length,
											)
										}
									/>
								</div>

								<div className="prompt-editor-field">
									<div className="prompt-editor-field-label">
										<span>自我介绍提示词</span>
										<span>仅用于自我介绍的那一次请求</span>
									</div>
									<p className="prompt-editor-field-description">
										适合写面试场景、身份口径、回答结构和需要优先强调的经历。
									</p>
									<Textarea
										value={interviewPromptDraft}
										onChange={(e) => {
											setInterviewPromptDraft(e.target.value);
										}}
										placeholder="请输入自我介绍专用提示词..."
										className="prompt-editor-textarea prompt-editor-textarea-compact"
										style={{
											scrollbarWidth: "none",
										}}
										rows={7}
									/>
								</div>

								<div className="prompt-editor-note">
									每次请求只会携带一份提示词。开场自我介绍请求使用自我介绍提示词，之后的请求继续使用常规提示词。
								</div>

								<div className="prompt-editor-actions">
									<Button
										type="button"
										variant="ghost"
										className="prompt-editor-secondary-button"
										onClick={handlePromptReset}
										disabled={isUsingDefaultPrompt}
									>
										<RotateCcw className="h-4 w-4" />
										恢复默认
									</Button>
									<div className="prompt-editor-actions-right">
										<Button
											type="button"
											variant="ghost"
											className="prompt-editor-tertiary-button"
											onClick={() => setIsPromptDialogOpen(false)}
										>
											取消
										</Button>
										<Button
											type="button"
											className="prompt-editor-primary-button"
											onClick={handlePromptSave}
											disabled={!hasPromptChanges}
										>
											<Save className="h-4 w-4" />
											保存提示词
										</Button>
									</div>
								</div>
							</div>
						</div>
					</div>
				</DialogContent>
			</Dialog>
		</>
	);
}
