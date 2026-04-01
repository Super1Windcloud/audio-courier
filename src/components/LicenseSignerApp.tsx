import { invoke } from "@tauri-apps/api/core";
import { Copy, FileSignature } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button.tsx";
import { Input } from "@/components/ui/input.tsx";
import { Textarea } from "@/components/ui/textarea.tsx";
import { copyText } from "@/lib/clipboard.ts";
import { logError, logInfo } from "@/lib/logger.ts";
import type {
	ActivationRequest,
	SignedLicense,
	SignerStatus,
} from "@/types/license.ts";

function defaultExpiryValue() {
	return "2099-12-31T23:59";
}

function oneYearExpiryValue() {
	const nextYear = new Date();
	nextYear.setFullYear(nextYear.getFullYear() + 1);
	nextYear.setSeconds(0, 0);
	const offset = nextYear.getTimezoneOffset();
	const local = new Date(nextYear.getTime() - offset * 60_000);
	return local.toISOString().slice(0, 16);
}

function toUtcIsoString(value: string) {
	const date = new Date(value);
	if (Number.isNaN(date.getTime())) {
		throw new Error("过期时间格式无效");
	}
	return date.toISOString();
}

function parseActivationRequest(value: string) {
	try {
		return JSON.parse(value) as ActivationRequest;
	} catch {
		return null;
	}
}

interface LicenseSignerAppProps {
	embedded?: boolean;
}

export function LicenseSignerApp({
	embedded = false,
}: LicenseSignerAppProps = {}) {
	const [requestJson, setRequestJson] = useState("");
	const [userId, setUserId] = useState("");
	const [expiresAt, setExpiresAt] = useState(defaultExpiryValue);
	const [isLifetime, setIsLifetime] = useState(true);
	const [maxVersion, setMaxVersion] = useState("1.9.99");
	const [featuresInput, setFeaturesInput] = useState("pro");
	const [licenseJson, setLicenseJson] = useState("");
	const [signerStatus, setSignerStatus] = useState<SignerStatus | null>(null);
	const [isSigning, setIsSigning] = useState(false);
	const [isLoadingStatus, setIsLoadingStatus] = useState(false);

	let parsedRequest: ActivationRequest | null = null;
	if (requestJson.trim()) {
		parsedRequest = parseActivationRequest(requestJson);
	}

	const loadSignerStatus = useCallback(async () => {
		setIsLoadingStatus(true);
		logInfo("license-signer load status");
		try {
			const status = await invoke<SignerStatus>("get_signer_status");
			setSignerStatus(status);
			logInfo(
				`license-signer status loaded allowed=${status.isAllowed} configured=${status.isConfigured}`,
			);
		} catch (error) {
			logError("license-signer load status failed", error);
			toast.error(String(error));
		} finally {
			setIsLoadingStatus(false);
		}
	}, []);

	useEffect(() => {
		loadSignerStatus().catch(() => {});
	}, [loadSignerStatus]);

	const handleSign = async () => {
		if (!requestJson.trim()) {
			toast.warning("请先粘贴 activation_request.json");
			return;
		}

		const nextUserId = userId.trim() || parsedRequest?.userId?.trim();
		if (!nextUserId) {
			toast.warning("缺少 userId");
			return;
		}

		setIsSigning(true);
		logInfo("license-signer start sign request");
		try {
			const result = await invoke<SignedLicense>("sign_activation_license", {
				rawRequest: requestJson,
				userId: nextUserId,
				expiresAt: toUtcIsoString(expiresAt),
				maxVersion: maxVersion.trim(),
				features: featuresInput
					.split(",")
					.map((item) => item.trim())
					.filter(Boolean),
			});
			const content = JSON.stringify(result, null, 2);
			setLicenseJson(content);
			const copied = await copyText(content);
			logInfo(`license-signer sign succeeded licenseId=${result.licenseId}`);
			toast.success(
				copied
					? "许可证已签发并复制到剪贴板"
					: "许可证已签发，请手动复制下方内容",
			);
		} catch (error) {
			logError("license-signer sign failed", error);
			toast.error(String(error));
		} finally {
			setIsSigning(false);
		}
	};

	const handleCopyLicense = async () => {
		if (!licenseJson) {
			toast.warning("当前没有已签发的 license.json");
			return;
		}
		const copied = await copyText(licenseJson);
		if (!copied) {
			toast.warning("当前环境不允许自动复制，请手动复制许可证内容");
			return;
		}
		logInfo("license-signer copied signed license");
		toast.success("许可证已复制");
	};

	if (signerStatus && !signerStatus.isAllowed) {
		return (
			<div
				className={`${embedded ? "text-white" : "min-h-screen bg-[radial-gradient(circle_at_top,#2f1f1f_0%,#141414_55%,#080808_100%)] px-6 py-6 text-white"}`}
			>
				<div
					className={`mx-auto flex ${embedded ? "max-w-none" : "min-h-[70vh] max-w-3xl"} items-center justify-center`}
				>
					<div className="w-full rounded-3xl border border-red-400/15 bg-black/30 p-8 backdrop-blur">
						<p className="text-xs uppercase tracking-[0.35em] text-red-200/70">
							Signer Locked
						</p>
						<h1 className="mt-3 text-3xl font-semibold">
							当前机器不能打开签名器
						</h1>
						<p className="mt-4 text-sm text-slate-300">{signerStatus.reason}</p>
						<p className="mt-4 text-xs text-slate-400">
							设备摘要: {signerStatus.currentDeviceHint}
						</p>
						<p className="mt-2 break-all text-xs text-slate-500">
							设备指纹: {signerStatus.currentDeviceFingerprint ?? "unknown"}
						</p>
					</div>
				</div>
			</div>
		);
	}

	return (
		<div
			className={`${embedded ? "min-h-0 text-white" : "min-h-screen bg-[radial-gradient(circle_at_top,#21405d_0%,#0b1322_45%,#050814_100%)] px-6 py-6 text-white"}`}
		>
			<div
				className={`mx-auto flex ${embedded ? "max-w-none pb-2" : "max-w-6xl"} flex-col gap-6`}
			>
				<div className="flex items-center justify-between rounded-3xl border border-cyan-400/15 bg-black/20 px-6 py-5 backdrop-blur">
					<div>
						<p className="text-xs uppercase tracking-[0.35em] text-cyan-200/70">
							Offline License Signer
						</p>
						<h1 className="mt-2 text-3xl font-semibold text-white">
							激活许可签名器
						</h1>
						<p className="mt-2 max-w-2xl text-sm text-slate-300">
							这个窗口只用于读取 activation request、填充授权参数并输出已签名的
							license.json。不要把带私钥的构建分发给最终用户。
						</p>
					</div>
					<Button
						type="button"
						variant="outline"
						className="border-cyan-300/30 bg-cyan-300/10 text-cyan-50 hover:bg-cyan-300/20"
						onClick={loadSignerStatus}
						disabled={isLoadingStatus}
					>
						<FileSignature className="size-4" />
						{isLoadingStatus ? "检查中..." : "检查签名配置"}
					</Button>
				</div>

				<div className="grid gap-6 xl:grid-cols-[1.15fr_0.85fr]">
					<div className="grid gap-6">
						<section className="rounded-3xl border border-white/10 bg-white/5 p-5 backdrop-blur">
							<div className="mb-4 flex items-center justify-between">
								<div>
									<h2 className="text-lg font-medium">激活请求</h2>
									<p className="text-sm text-slate-300">
										把用户从客户端导出的 activation_request.json 粘贴到这里。
									</p>
								</div>
								{parsedRequest ? (
									<span className="rounded-full bg-emerald-500/15 px-3 py-1 text-xs text-emerald-200">
										请求已解析
									</span>
								) : (
									<span className="rounded-full bg-amber-500/15 px-3 py-1 text-xs text-amber-100">
										等待有效 JSON
									</span>
								)}
							</div>
							<Textarea
								value={requestJson}
								onChange={(event) => {
									setRequestJson(event.target.value);
									if (!userId.trim()) {
										const request = parseActivationRequest(event.target.value);
										setUserId(request?.userId ?? "");
									}
								}}
								rows={9}
								className="border-white/10 bg-slate-950/80 font-mono text-xs text-slate-100"
								placeholder="粘贴 activation_request.json"
							/>
						</section>
					</div>

					<div className="grid gap-6">
						<section className="rounded-3xl border border-white/10 bg-white/5 p-5 backdrop-blur">
							<div className="mb-4 flex items-center justify-between">
								<div>
									<h2 className="text-lg font-medium">签名环境</h2>
									<p className="text-sm text-slate-300">
										需要在运行这个窗口的环境里配置 LICENSE_PRIVATE_KEY。
									</p>
								</div>
								<span
									className={`rounded-full px-3 py-1 text-xs ${
										signerStatus?.isAllowed && signerStatus?.isConfigured
											? "bg-emerald-500/15 text-emerald-200"
											: "bg-amber-500/15 text-amber-100"
									}`}
								>
									{signerStatus?.isAllowed && signerStatus?.isConfigured
										? "已配置"
										: "未配置"}
								</span>
							</div>
							<p className="text-sm text-slate-200">
								{signerStatus?.reason ?? "点击上方按钮检查签名器配置"}
							</p>
							<p className="mt-3 text-xs text-slate-400">
								设备摘要: {signerStatus?.currentDeviceHint ?? "unknown"}
							</p>
							<p className="mt-3 break-all text-xs text-slate-400">
								设备指纹: {signerStatus?.currentDeviceFingerprint ?? "unknown"}
							</p>
							<p className="mt-3 break-all text-xs text-slate-500">
								公钥: {signerStatus?.publicKey ?? "unknown"}
							</p>
						</section>

						<section className="rounded-3xl border border-white/10 bg-white/5 p-5 backdrop-blur">
							<div className="mb-4">
								<h2 className="text-lg font-medium">授权参数</h2>
								<p className="text-sm text-slate-300">
									签名时会把这些字段写入最终许可证。
								</p>
							</div>
							<div className="grid gap-4">
								<div className="grid gap-2">
									<label
										className="text-sm font-medium"
										htmlFor="signer-user-id"
									>
										用户标识
									</label>
									<Input
										id="signer-user-id"
										value={userId}
										onChange={(event) => setUserId(event.target.value)}
										className="border-white/10 bg-white/5 text-white"
										placeholder="customer_001"
									/>
								</div>
								<div className="grid gap-2">
									<label
										className="text-sm font-medium"
										htmlFor="signer-expires-at"
									>
										过期时间
									</label>
									<Input
										id="signer-expires-at"
										type="datetime-local"
										value={expiresAt}
										onChange={(event) => {
											setIsLifetime(false);
											setExpiresAt(event.target.value);
										}}
										className="border-white/10 bg-white/5 text-white"
										disabled={isLifetime}
									/>
									<div className="flex gap-2">
										<Button
											type="button"
											variant={isLifetime ? "default" : "outline"}
											size="sm"
											className={
												isLifetime
													? "bg-emerald-300 text-slate-950 hover:bg-emerald-200"
													: "border-white/10 bg-white/5 text-white"
											}
											onClick={() => {
												setIsLifetime(true);
												setExpiresAt(defaultExpiryValue());
											}}
										>
											永久授权
										</Button>
										<Button
											type="button"
											variant={!isLifetime ? "default" : "outline"}
											size="sm"
											className={
												!isLifetime
													? "bg-cyan-300 text-slate-950 hover:bg-cyan-200"
													: "border-white/10 bg-white/5 text-white"
											}
											onClick={() => {
												setIsLifetime(false);
												setExpiresAt(oneYearExpiryValue());
											}}
										>
											一年期
										</Button>
									</div>
									<p className="text-xs text-slate-400">
										永久授权默认写入 2099-12-31 23:59。
									</p>
								</div>
								<div className="grid gap-2">
									<label
										className="text-sm font-medium"
										htmlFor="signer-max-version"
									>
										最大版本
									</label>
									<Input
										id="signer-max-version"
										value={maxVersion}
										onChange={(event) => setMaxVersion(event.target.value)}
										className="border-white/10 bg-white/5 text-white"
										placeholder="1.9.99"
									/>
								</div>
								<div className="grid gap-2">
									<label
										className="text-sm font-medium"
										htmlFor="signer-features"
									>
										功能列表
									</label>
									<Input
										id="signer-features"
										value={featuresInput}
										onChange={(event) => setFeaturesInput(event.target.value)}
										className="border-white/10 bg-white/5 text-white"
										placeholder="pro, stt, export"
									/>
									<p className="text-xs text-slate-400">
										多个功能用英文逗号分隔。
									</p>
								</div>
								<Button
									type="button"
									className="mt-2 bg-cyan-300 text-slate-950 hover:bg-cyan-200"
									onClick={handleSign}
									disabled={isSigning}
								>
									{isSigning ? "签名中..." : "签发 license.json"}
								</Button>
							</div>
						</section>

						<section className="rounded-3xl border border-white/10 bg-white/5 p-5 text-sm text-slate-300 backdrop-blur">
							<h2 className="text-lg font-medium text-white">请求预览</h2>
							<p className="mt-3">用户: {parsedRequest?.userId ?? "-"}</p>
							<p className="mt-2">版本: {parsedRequest?.appVersion ?? "-"}</p>
							<p className="mt-2">
								设备摘要: {parsedRequest?.deviceHint ?? "-"}
							</p>
							<p className="mt-2 break-all">
								指纹: {parsedRequest?.deviceFingerprint ?? "-"}
							</p>
						</section>
					</div>
				</div>

				<section className="rounded-3xl border border-white/10 bg-white/5 p-5 backdrop-blur">
					<div className="mb-4">
						<h2 className="text-lg font-medium">签发结果</h2>
						<p className="text-sm text-slate-300">
							签名完成后会输出完整的 license.json，可直接发给用户。
						</p>
					</div>
					<div className="mb-3 flex justify-end">
						<Button
							type="button"
							variant="outline"
							className="border-white/10 bg-white/5 text-white"
							onClick={handleCopyLicense}
						>
							<Copy className="size-4" />
							复制 license.json
						</Button>
					</div>
					<Textarea
						value={licenseJson}
						readOnly
						rows={9}
						className="border-white/10 bg-slate-950/80 font-mono text-xs text-slate-100"
						placeholder="签发后这里会出现 license.json"
					/>
				</section>
			</div>
		</div>
	);
}
