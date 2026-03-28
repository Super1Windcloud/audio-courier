import { invoke } from "@tauri-apps/api/core";
import { MoreVertical, RotateCcw, Save, Sparkles } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";
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
import { HOTKEYS, MODEL_OPTIONS, type ModelOption } from "@/types/llm.ts";

export function MoreMenu() {
	const [currentModel, setCurrentModel] =
		useState<ModelOption>("siliconflow_pro");
	const appState = useAppStateStore();
	const [audioChannels, setAudioChannels] = useState<string[]>([]);
	const [isDialogOpen, setIsDialogOpen] = useState(false);
	const [isUpdating, setIsUpdating] = useState(false);
	const [promptDraft, setPromptDraft] = useState(appState.llmPrompt);
	const TRANSCRIBE_VENDORS: TranscribeVendor[] = [
		"assemblyai",
		"deepgram",
		"gladia",
		"revai",
		"speechmatics",
	];
	const VENDOR_LABELS: Record<TranscribeVendor, string> = {
		assemblyai: "AssemblyAI(English)",
		deepgram: "DeepGram",
		gladia: "Gladia",
		revai: "RevAI",
		speechmatics: "Speechmatics",
	};
	const UI_OPACITY_OPTIONS = [
		100, 95, 90, 85, 80, 75, 70, 65, 60, 55, 50, 45, 40, 35, 30,
	];
	const UI_TEXT_TONE_OPTIONS: { value: UiTextTone; label: string }[] = [
		{ value: "light", label: "浅色文字" },
		{ value: "dark", label: "深色文字" },
	];
	const defaultPrompt = import.meta.env.VITE_PROMPT || "";
	const promptCharacterCount = promptDraft.length;
	const hasPromptChanges = promptDraft !== appState.llmPrompt;
	const isUsingDefaultPrompt = promptDraft === defaultPrompt;

	useEffect(() => {
		invoke("get_audio_stream_devices_names").then((result) => {
			if (typeof result === "object" && Array.isArray(result)) {
				// console.log("audio devices ", result);
				setAudioChannels(result);
				appState.updateCurrentAudioChannel(result[0]);
			} else {
				toast.error("No audio streams output device  found");
			}
		});
	}, [appState.updateCurrentAudioChannel]);

	useEffect(() => {
		appState.updateCurrentSelectedModel(currentModel);
		console.log(currentModel);
	}, [currentModel, appState.updateCurrentSelectedModel]);

	useEffect(() => {
		if (!isDialogOpen) {
			return;
		}

		setPromptDraft(appState.llmPrompt);
	}, [appState.llmPrompt, isDialogOpen]);

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
		setIsDialogOpen(false);
		toast.success("提示词已保存");
	};

	const handlePromptReset = () => {
		setPromptDraft(defaultPrompt);
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
						onClick={() => setIsDialogOpen(true)}
						className="data-[highlighted]:bg-gray-500"
					>
						提示词
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
									onClick={() => setCurrentModel(model)}
								>
									{model}
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
									{VENDOR_LABELS[vendor]}
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
			<Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
				<DialogContent className="prompt-editor-dialog border-none p-0 text-white sm:max-w-[680px]">
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
									塑造回答语气、深度与追问节奏
								</DialogTitle>
								<DialogDescription className="prompt-editor-description">
									这里定义系统提示词。保存后会用于后续消息，不会回写历史回答。
								</DialogDescription>
							</DialogHeader>

							<div className="prompt-editor-status-row">
								<div className="prompt-editor-chip">
									{hasPromptChanges ? "未保存更改" : "当前已同步"}
								</div>
								<div className="prompt-editor-chip prompt-editor-chip-muted">
									{promptCharacterCount} 字符
								</div>
							</div>

							<div className="prompt-editor-field">
								<div className="prompt-editor-field-label">
									<span>提示词内容</span>
									<span>支持多行编辑</span>
								</div>
								<Textarea
									value={promptDraft}
									onChange={(e) => {
										setPromptDraft(e.target.value);
									}}
									placeholder="请输入提示词..."
									className="prompt-editor-textarea"
									autoFocus
									style={{
										scrollbarWidth: "none",
									}}
									rows={10}
									onFocus={(e) =>
										e.currentTarget.setSelectionRange(
											e.currentTarget.value.length,
											e.currentTarget.value.length,
										)
									}
								/>
							</div>

							<div className="prompt-editor-note">
								建议在这里描述角色设定、回答风格、语言偏好和追问力度，避免把临时问题写进系统提示词。
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
										onClick={() => setIsDialogOpen(false)}
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
				</DialogContent>
			</Dialog>
		</>
	);
}
