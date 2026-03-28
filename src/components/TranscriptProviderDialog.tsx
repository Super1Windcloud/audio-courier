import { RotateCcw, Save } from "lucide-react";
import { type ReactNode, useState } from "react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button.tsx";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog.tsx";
import { Input } from "@/components/ui/input.tsx";
import useAppStateStore from "@/stores";
import {
	createDefaultTranscriptProviderSettings,
	getTranscriptProviderStatus,
	type TranscriptProviderSettings,
} from "@/types/provider.ts";

function LabeledField({
	label,
	value,
	onChange,
	placeholder,
	description,
	status,
}: {
	label: string;
	value: string;
	onChange: (value: string) => void;
	placeholder?: string;
	description?: string;
	status?: string;
}) {
	return (
		<div className="grid gap-2">
			<div className="flex items-center justify-between gap-3">
				<span className="text-sm font-medium text-white">{label}</span>
				{status ? (
					<span className="rounded-full border border-white/10 bg-white/8 px-2.5 py-1 text-[11px] text-slate-200">
						{status}
					</span>
				) : null}
			</div>
			<Input
				value={value}
				onChange={(event) => onChange(event.target.value)}
				placeholder={placeholder}
				className="border-white/10 bg-white/5 text-white placeholder:text-slate-500"
			/>
			{description ? (
				<span className="text-xs leading-5 text-slate-400">{description}</span>
			) : null}
		</div>
	);
}

function Section({
	title,
	description,
	children,
}: {
	title: string;
	description: string;
	children: ReactNode;
}) {
	return (
		<section className="rounded-2xl border border-white/10 bg-white/5 p-4">
			<div className="mb-4">
				<h3 className="text-base font-medium text-white">{title}</h3>
				<p className="mt-1 text-sm leading-6 text-slate-300">{description}</p>
			</div>
			<div className="grid gap-4">{children}</div>
		</section>
	);
}

export function TranscriptProviderDialog({
	open,
	onOpenChange,
}: {
	open: boolean;
	onOpenChange: (open: boolean) => void;
}) {
	const settings = useAppStateStore(
		(state) => state.transcriptProviderSettings,
	);
	const updateSettings = useAppStateStore(
		(state) => state.updateTranscriptProviderSettings,
	);
	const [draft, setDraft] = useState<TranscriptProviderSettings>(settings);

	const hasChanges = JSON.stringify(draft) !== JSON.stringify(settings);

	return (
		<Dialog
			open={open}
			onOpenChange={(nextOpen) => {
				if (nextOpen) {
					setDraft(settings);
				}
				onOpenChange(nextOpen);
			}}
		>
			<DialogContent className="border-white/10 bg-slate-950 text-white sm:max-w-4xl">
				<DialogHeader>
					<DialogTitle>转录供应商 API 配置</DialogTitle>
					<DialogDescription className="text-slate-300">
						这里的配置只影响语音转录供应商，不影响大模型对话。API Key
						留空时会回退到 dev 模式下的 `.env` 或 production 模式下内置的
						`.env.local` 预设；语言、模型和实时地址会使用这里的值覆盖默认行为。
					</DialogDescription>
				</DialogHeader>

				<div className="max-h-[72vh] space-y-4 overflow-y-auto pr-1">
					<Section title="Deepgram" description="可配置 API Key 和语言代码。">
						<div className="grid gap-4 md:grid-cols-2">
							<LabeledField
								label="Deepgram API Key"
								value={draft.deepgramApiKey}
								onChange={(value) =>
									setDraft((current) => ({ ...current, deepgramApiKey: value }))
								}
								placeholder={getTranscriptProviderStatus(draft.deepgramApiKey)}
								status={getTranscriptProviderStatus(draft.deepgramApiKey)}
							/>
							<LabeledField
								label="Deepgram Language"
								value={draft.deepgramLanguage}
								onChange={(value) =>
									setDraft((current) => ({
										...current,
										deepgramLanguage: value,
									}))
								}
								placeholder="zh"
							/>
						</div>
					</Section>

					<Section
						title="AssemblyAI"
						description="目前只需要 AssemblyAI 的流式 API Key。"
					>
						<LabeledField
							label="AssemblyAI API Key"
							value={draft.assemblyApiKey}
							onChange={(value) =>
								setDraft((current) => ({ ...current, assemblyApiKey: value }))
							}
							placeholder={getTranscriptProviderStatus(draft.assemblyApiKey)}
							status={getTranscriptProviderStatus(draft.assemblyApiKey)}
						/>
					</Section>

					<Section
						title="Gladia"
						description="支持自定义 API Key、语言和模型名。"
					>
						<div className="grid gap-4 md:grid-cols-3">
							<LabeledField
								label="Gladia API Key"
								value={draft.gladiaApiKey}
								onChange={(value) =>
									setDraft((current) => ({ ...current, gladiaApiKey: value }))
								}
								placeholder={getTranscriptProviderStatus(draft.gladiaApiKey)}
								status={getTranscriptProviderStatus(draft.gladiaApiKey)}
							/>
							<LabeledField
								label="Gladia Language"
								value={draft.gladiaLanguage}
								onChange={(value) =>
									setDraft((current) => ({
										...current,
										gladiaLanguage: value,
									}))
								}
								placeholder="zh"
							/>
							<LabeledField
								label="Gladia Model"
								value={draft.gladiaModel}
								onChange={(value) =>
									setDraft((current) => ({ ...current, gladiaModel: value }))
								}
								placeholder="solaria-1"
							/>
						</div>
					</Section>

					<Section
						title="Speechmatics"
						description="支持 API Key、语言和实时接入 URL。"
					>
						<div className="grid gap-4 md:grid-cols-2">
							<LabeledField
								label="Speechmatics API Key"
								value={draft.speechmaticsApiKey}
								onChange={(value) =>
									setDraft((current) => ({
										...current,
										speechmaticsApiKey: value,
									}))
								}
								placeholder={getTranscriptProviderStatus(
									draft.speechmaticsApiKey,
								)}
								status={getTranscriptProviderStatus(draft.speechmaticsApiKey)}
							/>
							<LabeledField
								label="Speechmatics Language"
								value={draft.speechmaticsLanguage}
								onChange={(value) =>
									setDraft((current) => ({
										...current,
										speechmaticsLanguage: value,
									}))
								}
								placeholder="cmn"
							/>
							<div className="md:col-span-2">
								<LabeledField
									label="Speechmatics RT URL"
									value={draft.speechmaticsRtUrl}
									onChange={(value) =>
										setDraft((current) => ({
											...current,
											speechmaticsRtUrl: value,
										}))
									}
									placeholder="wss://eu2.rt.speechmatics.com/v2/"
								/>
							</div>
						</div>
					</Section>

					<Section title="RevAI" description="支持 API Key、语言和 metadata。">
						<div className="grid gap-4 md:grid-cols-2">
							<LabeledField
								label="RevAI API Key"
								value={draft.revaiApiKey}
								onChange={(value) =>
									setDraft((current) => ({ ...current, revaiApiKey: value }))
								}
								placeholder={getTranscriptProviderStatus(draft.revaiApiKey)}
								status={getTranscriptProviderStatus(draft.revaiApiKey)}
							/>
							<LabeledField
								label="RevAI Language"
								value={draft.revaiLanguage}
								onChange={(value) =>
									setDraft((current) => ({ ...current, revaiLanguage: value }))
								}
								placeholder="cmn"
							/>
							<div className="md:col-span-2">
								<LabeledField
									label="RevAI Metadata"
									value={draft.revaiMetadata}
									onChange={(value) =>
										setDraft((current) => ({
											...current,
											revaiMetadata: value,
										}))
									}
									placeholder="optional metadata"
								/>
							</div>
						</div>
					</Section>
				</div>

				<div className="flex items-center justify-between gap-3 border-t border-white/10 pt-4">
					<Button
						type="button"
						variant="ghost"
						className="text-slate-200 hover:bg-white/10 hover:text-white"
						onClick={() => {
							setDraft(createDefaultTranscriptProviderSettings());
						}}
					>
						<RotateCcw className="size-4" />
						恢复默认
					</Button>
					<div className="flex items-center gap-2">
						<Button
							type="button"
							variant="ghost"
							className="text-slate-200 hover:bg-white/10 hover:text-white"
							onClick={() => onOpenChange(false)}
						>
							取消
						</Button>
						<Button
							type="button"
							className="bg-emerald-300 text-slate-950 hover:bg-emerald-200"
							disabled={!hasChanges}
							onClick={() => {
								updateSettings(draft);
								onOpenChange(false);
								toast.success("转录供应商配置已保存");
							}}
						>
							<Save className="size-4" />
							保存配置
						</Button>
					</div>
				</div>
			</DialogContent>
		</Dialog>
	);
}
