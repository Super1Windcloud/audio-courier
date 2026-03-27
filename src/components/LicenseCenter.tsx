import { invoke } from "@tauri-apps/api/core";
import { Copy, KeyRound } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button.tsx";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
	DialogTrigger,
} from "@/components/ui/dialog.tsx";
import { Input } from "@/components/ui/input.tsx";
import { Textarea } from "@/components/ui/textarea.tsx";
import useAppStateStore from "@/stores";
import type { ActivationRequest, LicenseStatus } from "@/types/license.ts";

export function LicenseCenter() {
	const licenseStatus = useAppStateStore((state) => state.licenseStatus);
	const updateLicenseStatus = useAppStateStore(
		(state) => state.updateLicenseStatus,
	);
	const [open, setOpen] = useState(false);
	const [userId, setUserId] = useState(licenseStatus?.userId ?? "customer");
	const [requestJson, setRequestJson] = useState("");
	const [licenseJson, setLicenseJson] = useState("");
	const [isLoadingRequest, setIsLoadingRequest] = useState(false);
	const [isImporting, setIsImporting] = useState(false);

	const handleGenerateRequest = async () => {
		setIsLoadingRequest(true);
		try {
			const request = await invoke<ActivationRequest>(
				"get_activation_request",
				{
					userId: userId.trim() || null,
				},
			);
			const content = JSON.stringify(request, null, 2);
			setRequestJson(content);
			await navigator.clipboard.writeText(content);
			toast.success("设备请求码已生成并复制到剪贴板");
		} catch (error) {
			toast.error(String(error));
		} finally {
			setIsLoadingRequest(false);
		}
	};

	const handleCopyRequest = async () => {
		if (!requestJson) {
			toast.warning("请先生成设备请求码");
			return;
		}
		await navigator.clipboard.writeText(requestJson);
		toast.success("设备请求码已复制");
	};

	const handleImportLicense = async () => {
		if (!licenseJson.trim()) {
			toast.warning("请粘贴许可证 JSON");
			return;
		}

		setIsImporting(true);
		try {
			const nextStatus = await invoke<LicenseStatus>("import_license", {
				rawLicense: licenseJson,
			});
			updateLicenseStatus(nextStatus);
			toast.success("许可证已激活");
			setOpen(false);
		} catch (error) {
			toast.error(String(error));
		} finally {
			setIsImporting(false);
		}
	};

	return (
		<Dialog open={open} onOpenChange={setOpen}>
			<DialogTrigger asChild>
				<button
					type="button"
					className="inline-flex h-8 items-center gap-2 rounded-md px-3 text-sm text-gray-200 transition-colors hover:bg-white/10 hover:text-white"
				>
					<KeyRound className="size-4" />
					许可证
				</button>
			</DialogTrigger>
			<DialogContent className="border-white/10 bg-slate-950 text-white sm:max-w-2xl">
				<DialogHeader>
					<DialogTitle>离线激活</DialogTitle>
					<DialogDescription className="text-slate-300">
						先生成设备请求码发给你自己签名，再把返回的许可证 JSON
						粘贴到这里导入。
					</DialogDescription>
				</DialogHeader>

				<div className="grid gap-4">
					<div className="rounded-xl border border-white/10 bg-white/5 p-4">
						<div className="mb-2 flex items-center justify-between">
							<div>
								<p className="text-sm font-medium">当前状态</p>
								<p className="text-xs text-slate-300">
									{licenseStatus?.reason ?? "未加载"}
								</p>
							</div>
							<span
								className={`rounded-full px-3 py-1 text-xs ${licenseStatus?.isValid ? "bg-emerald-500/20 text-emerald-300" : "bg-amber-500/20 text-amber-200"}`}
							>
								{licenseStatus?.isValid ? "已激活" : "未激活"}
							</span>
						</div>
						<p className="text-xs text-slate-400">
							设备摘要: {licenseStatus?.deviceHint ?? "unknown"}
						</p>
						<p className="mt-1 text-xs text-slate-400 break-all">
							指纹: {licenseStatus?.deviceFingerprint ?? "unknown"}
						</p>
					</div>

					<div className="grid gap-2">
						<label className="text-sm font-medium" htmlFor="license-user-id">
							用户标识
						</label>
						<Input
							id="license-user-id"
							value={userId}
							onChange={(event) => setUserId(event.target.value)}
							className="border-white/10 bg-white/5 text-white"
							placeholder="customer_001"
						/>
					</div>

					<div className="grid gap-2">
						<div className="flex items-center justify-between">
							<p className="text-sm font-medium">设备请求码</p>
							<div className="flex gap-2">
								<Button
									type="button"
									variant="outline"
									size="sm"
									className="border-white/10 bg-white/5 text-white"
									onClick={handleCopyRequest}
								>
									<Copy className="size-4" />
									复制
								</Button>
								<Button
									type="button"
									size="sm"
									className="bg-cyan-500 text-slate-950 hover:bg-cyan-400"
									onClick={handleGenerateRequest}
									disabled={isLoadingRequest}
								>
									{isLoadingRequest ? "生成中..." : "生成请求码"}
								</Button>
							</div>
						</div>
						<Textarea
							value={requestJson}
							readOnly
							rows={8}
							className="border-white/10 bg-slate-900 text-xs text-slate-100"
							placeholder="生成后会自动复制到剪贴板"
						/>
					</div>

					<div className="grid gap-2">
						<p className="text-sm font-medium">导入许可证</p>
						<Textarea
							value={licenseJson}
							onChange={(event) => setLicenseJson(event.target.value)}
							rows={10}
							className="border-white/10 bg-slate-900 text-xs text-slate-100"
							placeholder="把你签发的 license.json 内容粘贴到这里"
						/>
						<Button
							type="button"
							className="bg-emerald-400 text-slate-950 hover:bg-emerald-300"
							onClick={handleImportLicense}
							disabled={isImporting}
						>
							{isImporting ? "校验中..." : "导入并激活"}
						</Button>
					</div>
				</div>
			</DialogContent>
		</Dialog>
	);
}
