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
					className="rounded p-1 transition-colors hover:bg-white/10"
					title="关于项目"
				>
					<Info className="h-5 w-5 text-slate-300" />
				</button>
			</DialogTrigger>
			<DialogContent className="border-white/10 bg-[linear-gradient(180deg,rgba(25,37,54,0.96)_0%,rgba(15,22,34,0.98)_100%)] text-slate-100 shadow-[0_24px_70px_rgba(3,8,20,0.45)] backdrop-blur-xl sm:max-w-[425px]">
				<DialogHeader>
					<DialogTitle className="flex items-center gap-2 text-2xl font-bold">
						<img src="/icon.png" alt="Logo" className="w-8 h-8" />
						Audio Courier
					</DialogTitle>
					<DialogDescription className="text-slate-300">
						实时语音转文字助手 (Real-time Speech-to-Text)
					</DialogDescription>
				</DialogHeader>
				<div className="grid gap-4 py-4">
					<div className="flex flex-col gap-2">
						<div className="flex justify-between items-center text-sm">
							<span className="font-medium">版本</span>
							<span className="text-slate-300">{version}</span>
						</div>
						<div className="flex justify-between items-center text-sm">
							<span className="font-medium">作者</span>
							<span className="text-slate-300">superwindcloud</span>
						</div>
						<div className="flex justify-between items-center text-sm">
							<span className="font-medium">许可证</span>
							<span className="text-slate-300">Apache-2.0</span>
						</div>
					</div>
					<div className="mt-2 border-t border-white/10 pt-4">
						<p className="text-sm leading-relaxed text-slate-300">
							Audio Courier 是一款专注于本地化使用的实时语音转文字应用。
							它支持多种转录后端，能够快速准确地将语音流转换为文本，
							并提供流畅的对话式交互体验。
						</p>
					</div>
					<div className="mt-2 flex items-center gap-2">
						<a
							href="https://github.com/super1windcloud/audio-courier"
							target="_blank"
							rel="noreferrer"
							className="flex items-center gap-2 text-sm text-cyan-300 transition-colors hover:text-cyan-200 hover:underline"
						>
							GitHub 仓库
						</a>
					</div>
				</div>
			</DialogContent>
		</Dialog>
	);
};
