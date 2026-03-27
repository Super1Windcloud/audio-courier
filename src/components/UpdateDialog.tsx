import type { Update } from "@tauri-apps/plugin-updater";
import { Download, Sparkles } from "lucide-react";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog.tsx";
import { Button } from "@/components/ui/button.tsx";
import { formatBytes } from "@/lib/updater.ts";

interface UpdateDialogProps {
	open: boolean;
	update: Update | null;
	isInstalling: boolean;
	progressTotalBytes: number;
	progressDownloadedBytes: number;
	onOpenChange: (open: boolean) => void;
	onInstall: () => void;
}

export function UpdateDialog({
	open,
	update,
	isInstalling,
	progressTotalBytes,
	progressDownloadedBytes,
	onOpenChange,
	onInstall,
}: UpdateDialogProps) {
	const progressPercent =
		progressTotalBytes > 0
			? Math.min(
					100,
					Math.round((progressDownloadedBytes / progressTotalBytes) * 100),
				)
			: 0;

	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="max-w-xl border-white/10 bg-[linear-gradient(180deg,#13243a_0%,#0a1320_100%)] p-0 text-white shadow-2xl shadow-cyan-950/40">
				<div className="rounded-[inherit] border border-white/10 bg-[radial-gradient(circle_at_top,#4ecdc41f_0%,transparent_48%),linear-gradient(180deg,#12243d_0%,#0a1320_100%)]">
					<DialogHeader className="space-y-0 px-7 py-6">
						<div className="inline-flex size-11 items-center justify-center rounded-2xl bg-cyan-300/12 text-cyan-200">
							<Sparkles className="size-5" />
						</div>
						<DialogTitle className="pt-4 text-2xl font-semibold tracking-tight text-white">
							发现新版本 {update?.version}
						</DialogTitle>
						<DialogDescription className="pt-2 text-sm leading-6 text-slate-300">
							{update?.body?.trim() || "检测到可用更新。安装后应用可能会退出并启动安装器。"}
						</DialogDescription>
					</DialogHeader>

					<div className="px-7 pb-4">
						<div className="rounded-2xl border border-white/8 bg-white/5 p-4">
							<div className="flex items-center justify-between text-sm text-slate-300">
								<span>更新包状态</span>
								<span>
									{progressTotalBytes > 0
										? `${formatBytes(progressDownloadedBytes)} / ${formatBytes(progressTotalBytes)}`
										: "等待开始"}
								</span>
							</div>
							<div className="mt-3 h-2 overflow-hidden rounded-full bg-white/10">
								<div
									className="h-full rounded-full bg-gradient-to-r from-cyan-300 via-sky-400 to-emerald-300 transition-all duration-300"
									style={{ width: `${progressPercent}%` }}
								/>
							</div>
							<div className="mt-3 text-xs text-slate-400">
								{isInstalling
									? progressTotalBytes > 0
										? `正在下载安装 ${progressPercent}%`
										: "正在准备下载安装"
									: "建议现在安装，避免继续运行旧版本。"}
							</div>
						</div>
					</div>

					<DialogFooter className="border-t border-white/8 px-7 py-5 sm:justify-between">
						<p className="text-xs text-slate-400">
							应用退出或弹出安装窗口属于正常行为。
						</p>
						<Button
							type="button"
							onClick={onInstall}
							disabled={isInstalling || !update}
							className="bg-cyan-300 text-slate-950 hover:bg-cyan-200"
						>
							<Download className="size-4" />
							{isInstalling ? "安装中..." : "立即更新"}
						</Button>
					</DialogFooter>
				</div>
			</DialogContent>
		</Dialog>
	);
}
