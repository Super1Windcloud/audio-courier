import { invoke } from "@tauri-apps/api/core";
import {
	BadgeCheck,
	Copy,
	KeyRound,
	MonitorSmartphone,
	ShieldCheck,
	Sparkles,
} from "lucide-react";
import { useEffect, useRef, useState } from "react";
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
import { copyText } from "@/lib/clipboard.ts";
import useAppStateStore from "@/stores";
import type { ActivationRequest, LicenseStatus } from "@/types/license.ts";

function formatDate(value: string | null) {
	if (!value) {
		return "未设置";
	}

	const date = new Date(value);
	if (Number.isNaN(date.getTime())) {
		return value;
	}

	return new Intl.DateTimeFormat("zh-CN", {
		year: "numeric",
		month: "2-digit",
		day: "2-digit",
		hour: "2-digit",
		minute: "2-digit",
	}).format(date);
}

function StatusHero({ licenseStatus }: { licenseStatus: LicenseStatus }) {
	const isHostSigner = licenseStatus.isHostSigner;
	const title = isHostSigner ? "宿主机已授权" : "许可证已激活";
	const subtitle = isHostSigner
		? "当前设备被标记为签名宿主机，可直接使用完整能力并打开签名器。"
		: "当前设备已完成离线授权校验，可以正常使用高级功能。";
	const badgeLabel = isHostSigner ? "Host Signer" : "Activated";
	const accentClass = isHostSigner
		? "from-amber-400/25 via-orange-300/10 to-transparent"
		: "from-emerald-400/25 via-cyan-300/10 to-transparent";
	const iconClass = isHostSigner
		? "bg-amber-300/15 text-amber-200 ring-1 ring-amber-200/25"
		: "bg-emerald-300/15 text-emerald-200 ring-1 ring-emerald-200/25";
	const statusClass = isHostSigner
		? "bg-amber-300/15 text-amber-100 ring-1 ring-amber-200/25"
		: "bg-emerald-300/15 text-emerald-100 ring-1 ring-emerald-200/25";

	return (
		<div
			className={`relative overflow-hidden rounded-3xl border border-white/12 bg-[radial-gradient(circle_at_top_left,_rgba(255,255,255,0.18),_transparent_42%),linear-gradient(135deg,_rgba(15,23,42,0.96),_rgba(6,12,24,0.98))] p-6`}
		>
			<div
				className={`pointer-events-none absolute inset-0 bg-gradient-to-br ${accentClass}`}
			/>
			<div className="relative flex flex-col gap-6">
				<div className="flex items-start justify-between gap-4">
					<div className="flex items-center gap-4">
						<div
							className={`flex size-14 items-center justify-center rounded-2xl backdrop-blur ${iconClass}`}
						>
							{isHostSigner ? (
								<MonitorSmartphone className="size-7" />
							) : (
								<ShieldCheck className="size-7" />
							)}
						</div>
						<div className="space-y-2">
							<div className="inline-flex items-center gap-2 rounded-full bg-white/8 px-3 py-1 text-[11px] tracking-[0.24em] text-slate-200 uppercase ring-1 ring-white/10">
								<Sparkles className="size-3.5" />
								{badgeLabel}
							</div>
							<div>
								<h3 className="text-2xl font-semibold tracking-tight text-white">
									{title}
								</h3>
								<p className="mt-1 max-w-xl text-sm leading-6 text-slate-300">
									{subtitle}
								</p>
							</div>
						</div>
					</div>
					<div
						className={`inline-flex items-center gap-2 rounded-full px-3 py-1 text-xs font-medium ${statusClass}`}
					>
						<BadgeCheck className="size-4" />
						{isHostSigner ? "宿主机放行" : "校验通过"}
					</div>
				</div>

				<div className="grid gap-3 md:grid-cols-3">
					<div className="rounded-2xl border border-white/10 bg-white/6 p-4 backdrop-blur-sm">
						<p className="text-xs uppercase tracking-[0.22em] text-slate-400">
							授权主体
						</p>
						<p className="mt-2 text-base font-medium text-white">
							{licenseStatus.userId ?? "本机授权"}
						</p>
						<p className="mt-1 text-xs text-slate-400">
							{licenseStatus.licenseId ?? "未写入 license.json"}
						</p>
					</div>
					<div className="rounded-2xl border border-white/10 bg-white/6 p-4 backdrop-blur-sm">
						<p className="text-xs uppercase tracking-[0.22em] text-slate-400">
							授权期限
						</p>
						<p className="mt-2 text-base font-medium text-white">
							{isHostSigner
								? "宿主机长期放行"
								: formatDate(licenseStatus.expiresAt)}
						</p>
						<p className="mt-1 text-xs text-slate-400">
							签发时间 {formatDate(licenseStatus.issuedAt)}
						</p>
					</div>
					<div className="rounded-2xl border border-white/10 bg-white/6 p-4 backdrop-blur-sm">
						<p className="text-xs uppercase tracking-[0.22em] text-slate-400">
							版本范围
						</p>
						<p className="mt-2 text-base font-medium text-white">
							{licenseStatus.maxVersion ?? "当前版本可用"}
						</p>
						<p className="mt-1 text-xs text-slate-400">
							当前版本 {licenseStatus.currentVersion}
						</p>
					</div>
				</div>
			</div>
		</div>
	);
}

export function LicenseCenter() {
	const licenseStatus = useAppStateStore((state) => state.licenseStatus);
	const updateLicenseStatus = useAppStateStore(
		(state) => state.updateLicenseStatus,
	);
	const [open, setOpen] = useState(false);
	const didAutoOpen = useRef(false);
	const [userId, setUserId] = useState(licenseStatus?.userId ?? "customer");
	const [requestJson, setRequestJson] = useState("");
	const [licenseJson, setLicenseJson] = useState("");
	const [isLoadingRequest, setIsLoadingRequest] = useState(false);
	const [isImporting, setIsImporting] = useState(false);
	const isAuthorized = Boolean(
		licenseStatus?.isValid || licenseStatus?.isHostSigner,
	);

	useEffect(() => {
		if (!licenseStatus || isAuthorized || didAutoOpen.current) {
			return;
		}
		didAutoOpen.current = true;
		setOpen(true);
	}, [isAuthorized, licenseStatus]);

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
			const copied = await copyText(content);
			toast.success(
				copied
					? "设备请求码已生成并复制到剪贴板"
					: "设备请求码已生成，请手动复制下方内容",
			);
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
		const copied = await copyText(requestJson);
		if (copied) {
			toast.success("设备请求码已复制");
			return;
		}
		toast.warning("当前环境不允许自动复制，请手动复制请求码");
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
					<DialogTitle>{isAuthorized ? "许可证状态" : "离线激活"}</DialogTitle>
					<DialogDescription className="text-slate-300">
						{isAuthorized
							? "当前设备已完成授权校验，这里展示许可证和设备绑定信息。"
							: "先生成设备请求码发给你自己签名，再把返回的许可证 JSON 粘贴到这里导入。"}
					</DialogDescription>
				</DialogHeader>

				<div className="grid gap-4">
					{licenseStatus && isAuthorized ? (
						<StatusHero licenseStatus={licenseStatus} />
					) : null}

					<div className="rounded-2xl border border-white/10 bg-white/5 p-4">
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
								{licenseStatus?.isHostSigner
									? "宿主机"
									: licenseStatus?.isValid
										? "已激活"
										: "未激活"}
							</span>
						</div>
						<p className="text-xs text-slate-400">
							设备摘要: {licenseStatus?.deviceHint ?? "unknown"}
						</p>
						<p className="mt-1 text-xs text-slate-400 break-all">
							指纹: {licenseStatus?.deviceFingerprint ?? "unknown"}
						</p>
					</div>

					{licenseStatus && isAuthorized ? (
						<div className="grid gap-4 md:grid-cols-[1.4fr_1fr]">
							<div className="rounded-2xl border border-white/10 bg-[linear-gradient(160deg,_rgba(255,255,255,0.08),_rgba(255,255,255,0.03))] p-5">
								<p className="text-sm font-medium text-white">授权详情</p>
								<div className="mt-4 grid gap-4 sm:grid-cols-2">
									<div>
										<p className="text-xs uppercase tracking-[0.2em] text-slate-400">
											授权功能
										</p>
										<div className="mt-3 flex flex-wrap gap-2">
											{licenseStatus.features.length > 0 ? (
												licenseStatus.features.map((feature) => (
													<span
														key={feature}
														className="rounded-full border border-emerald-300/20 bg-emerald-300/10 px-3 py-1 text-xs text-emerald-100"
													>
														{feature}
													</span>
												))
											) : (
												<span className="text-sm text-slate-400">未限制</span>
											)}
										</div>
									</div>
									<div>
										<p className="text-xs uppercase tracking-[0.2em] text-slate-400">
											最近校验
										</p>
										<p className="mt-3 text-sm text-slate-100">
											{formatDate(licenseStatus.checkedAt)}
										</p>
										<p className="mt-1 text-xs text-slate-400">
											{licenseStatus.reason}
										</p>
									</div>
								</div>
							</div>

							<div className="rounded-2xl border border-white/10 bg-white/4 p-5">
								<p className="text-sm font-medium text-white">设备绑定</p>
								<p className="mt-4 text-sm text-slate-200">
									{licenseStatus.deviceHint}
								</p>
								<p className="mt-3 break-all text-xs leading-6 text-slate-400">
									{licenseStatus.deviceFingerprint}
								</p>
							</div>
						</div>
					) : (
						<>
							<div className="grid gap-2">
								<label
									className="text-sm font-medium"
									htmlFor="license-user-id"
								>
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
						</>
					)}
				</div>
			</DialogContent>
		</Dialog>
	);
}
