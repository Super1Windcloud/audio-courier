import { FileSignature } from "lucide-react";
import { useEffect, useState } from "react";
import { LicenseSignerApp } from "@/components/LicenseSignerApp.tsx";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
	DialogTrigger,
} from "@/components/ui/dialog.tsx";
import { logInfo } from "@/lib/logger.ts";
import type { SignerStatus } from "@/types/license.ts";

interface LicenseSignerCenterProps {
	signerStatus: SignerStatus | null;
}

export function LicenseSignerCenter({
	signerStatus,
}: LicenseSignerCenterProps) {
	const [open, setOpen] = useState(false);

	useEffect(() => {
		if (!open) {
			return;
		}
		logInfo("license-signer dialog opened");
	}, [open]);

	if (!signerStatus?.isAllowed) {
		return null;
	}

	return (
		<Dialog open={open} onOpenChange={setOpen}>
			<DialogTrigger asChild>
				<button
					type="button"
					className="inline-flex h-8 items-center gap-2 rounded-md px-3 text-sm text-gray-200 transition-colors hover:bg-white/10 hover:text-white"
				>
					<FileSignature className="size-4" />
					签名器
				</button>
			</DialogTrigger>
			<DialogContent className="max-h-[88vh] overflow-hidden border-white/10 bg-slate-950 text-white sm:max-w-6xl">
				<DialogHeader>
					<DialogTitle>激活许可签名器</DialogTitle>
					<DialogDescription className="text-slate-300">
						这里直接在主应用里签发 license.json，不再创建新的 Tauri 窗口。
					</DialogDescription>
				</DialogHeader>
				<div className="overflow-y-auto pr-1">
					<LicenseSignerApp embedded />
				</div>
			</DialogContent>
		</Dialog>
	);
}
