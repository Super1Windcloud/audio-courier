import { useEffect, useState } from "react";
import { MODEL_OPTIONS, ModelOption } from "@/types/llm.ts";
import useAppStateStore from "@/stores";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSub,
	DropdownMenuSubContent,
	DropdownMenuSubTrigger,
	DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu.tsx";
import { MoreVertical } from "lucide-react";
import {
	Dialog,
	DialogContent,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog.tsx";
import { Input } from "@/components/ui/input.tsx";

export function MoreMenu() {
	const [currentModel, setCurrentModel] = useState<ModelOption>("siliconflow");
	const appState = useAppStateStore();
	const [audioChannels, setAudioChannels] = useState<string[]>([]);
	const [isDialogOpen, setIsDialogOpen] = useState(false);

	useEffect(() => {
		invoke("get_audio_stream_devices_name").then((result) => {
			if (typeof result === "object" && Array.isArray(result)) {
				console.log("audio devices ", result);
				setAudioChannels(result);
				appState.updateCurrentAudioChannel(result[0]);
			} else {
				toast.error("No audio streams found");
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
					<DropdownMenuItem className="flex items-center justify-between">
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

					<Input
						value={appState.llmPrompt}
						onChange={(e) => {
							e.currentTarget.setSelectionRange(
								e.currentTarget.value.length,
								e.currentTarget.value.length,
							);
							appState.updateLLMPrompt(e.target.value);
						}}
						placeholder="请输入提示词..."
						className="mt-2 w-full"
						autoFocus={false}
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
