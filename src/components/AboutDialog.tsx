import { getVersion } from "@tauri-apps/api/app";
import { Info } from "lucide-react";
import type React from "react";
import { useEffect, useState } from "react";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
	DialogTrigger,
} from "@/components/ui/dialog.tsx";

export const AboutDialog: React.FC = () => {
	const [version, setVersion] = useState<string>("1.0.0");

	useEffect(() => {
		getVersion().then(setVersion);
	}, []);

	return (
		<Dialog>
			<DialogTrigger asChild>
				<button
					type="button"
					className="p-1 hover:bg-gray-200 dark:hover:bg-gray-700 rounded transition-colors"
					title="关于项目"
				>
					<Info className="w-5 h-5 text-gray-500 dark:text-gray-400" />
				</button>
			</DialogTrigger>
			<DialogContent className="sm:max-w-[425px] bg-white dark:bg-gray-800 border-none">
				<DialogHeader>
					<DialogTitle className="flex items-center gap-2 text-2xl font-bold">
						<img src="/icon.png" alt="Logo" className="w-8 h-8" />
						Audio Courier
					</DialogTitle>
					<DialogDescription className="text-gray-500 dark:text-gray-400">
						实时语音转文字助手 (Real-time Speech-to-Text)
					</DialogDescription>
				</DialogHeader>
				<div className="grid gap-4 py-4">
					<div className="flex flex-col gap-2">
						<div className="flex justify-between items-center text-sm">
							<span className="font-medium">版本</span>
							<span className="text-gray-500">{version}</span>
						</div>
						<div className="flex justify-between items-center text-sm">
							<span className="font-medium">作者</span>
							<span className="text-gray-500">superwindcloud</span>
						</div>
						<div className="flex justify-between items-center text-sm">
							<span className="font-medium">许可证</span>
							<span className="text-gray-500">Apache-2.0</span>
						</div>
					</div>
					<div className="border-t pt-4 mt-2">
						<p className="text-sm text-gray-600 dark:text-gray-300 leading-relaxed">
							Audio Courier 是一款专注于本地化使用的实时语音转文字应用。
							它支持多种转录后端，能够快速准确地将语音流转换为文本，
							并提供流畅的对话式交互体验。
						</p>
					</div>
					<div className="flex items-center gap-2 mt-2">
						<a
							href="https://github.com/super1windcloud/audio-courier"
							target="_blank"
							rel="noreferrer"
							className="flex items-center gap-2 text-sm text-blue-500 hover:underline"
						>
							GitHub 仓库
						</a>
					</div>
				</div>
			</DialogContent>
		</Dialog>
	);
};
