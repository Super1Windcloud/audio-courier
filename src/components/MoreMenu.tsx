import { invoke } from "@tauri-apps/api/core";
import { MoreVertical } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";
import {
	Dialog,
	DialogContent,
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
import useAppStateStore, { TranscribeVendor } from "@/stores";
import { HOTKEYS, MODEL_OPTIONS, ModelOption } from "@/types/llm.ts";

export function MoreMenu() {
	const [currentModel, setCurrentModel] =
		useState<ModelOption>("siliconflow_free");
	const appState = useAppStateStore();
	const [audioChannels, setAudioChannels] = useState<string[]>([]);
	const [isDialogOpen, setIsDialogOpen] = useState(false);
	const TRANSCRIBE_VENDORS: TranscribeVendor[] = [
		"assemblyai",
		"deepgram",
		"gladia",
		"revai",
		"speechmatics",
	];
	const VENDOR_LABELS: Record<TranscribeVendor, string> = {
		assemblyai: "AssemblyAI",
		deepgram: "DeepGram",
		gladia: "GlaDia",
		revai: "RevAI",
		speechmatics: "Speechmatics",
	};

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
	}, []);

	useEffect(() => {
		appState.updateCurrentSelectedModel(currentModel);
		console.log(currentModel);
	}, [currentModel]);

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
							{audioChannels.map((devices, index) => (
								<DropdownMenuItem
									key={devices + index}
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
				</DropdownMenuContent>
			</DropdownMenu>
			<Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
				<DialogContent className="sm:max-w-[400px] bg-pink-200">
					<DialogHeader>
						<DialogTitle>输入提示词</DialogTitle>
					</DialogHeader>

					<Textarea
						value={appState.llmPrompt}
						onChange={(e) => {
							appState.updateLLMPrompt(e.target.value);
						}}
						placeholder="请输入提示词..."
						className="mt-2 w-full"
						autoFocus={true}
						style={{
							scrollbarWidth: "none",
						}}
						rows={6} // 默认高度，可自行调整
						onFocus={(e) =>
							e.currentTarget.setSelectionRange(
								e.currentTarget.value.length,
								e.currentTarget.value.length,
							)
						}
					/>
				</DialogContent>
			</Dialog>
		</>
	);
}
