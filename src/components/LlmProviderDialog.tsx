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
	createDefaultLlmProviderSettings,
	getLlmProviderStatus,
	type LlmProviderSettings,
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

export function LlmProviderDialog({
	open,
	onOpenChange,
}: {
	open: boolean;
	onOpenChange: (open: boolean) => void;
}) {
	const settings = useAppStateStore((state) => state.llmProviderSettings);
	const presets = useAppStateStore((state) => state.envProviderPresets.llm);
	const updateSettings = useAppStateStore(
		(state) => state.updateLlmProviderSettings,
	);
	const [draft, setDraft] = useState<LlmProviderSettings>(settings);

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
					<DialogTitle>大模型 API 配置</DialogTitle>
					<DialogDescription className="text-slate-300">
						这里的配置只影响大模型请求，不影响转录供应商。API Key 留空时会回退到
						dev 模式下的 `.env` 或 production 模式下内置的 `.env.local`
						预设；OpenAI、Gemini 和自定义兼容供应商还支持自定义 Base URL
						与模型名。
					</DialogDescription>
				</DialogHeader>

				<div className="max-h-[72vh] space-y-4 overflow-y-auto pr-1">
					<Section
						title="内置供应商 API Key"
						description="这些字段对应当前仓库已有的大模型供应商。只需要填 API Key。"
					>
						<div className="grid gap-4 md:grid-cols-2">
							<LabeledField
								label="SiliconFlow API Key"
								value={draft.siliconflowApiKey}
								onChange={(value) =>
									setDraft((current) => ({
										...current,
										siliconflowApiKey: value,
									}))
								}
								placeholder={getLlmProviderStatus(
									draft.siliconflowApiKey,
									presets.siliconflowApiKey,
								)}
								status={getLlmProviderStatus(
									draft.siliconflowApiKey,
									presets.siliconflowApiKey,
								)}
							/>
							<LabeledField
								label="Doubao API Key"
								value={draft.doubaoApiKey}
								onChange={(value) =>
									setDraft((current) => ({ ...current, doubaoApiKey: value }))
								}
								placeholder={getLlmProviderStatus(
									draft.doubaoApiKey,
									presets.doubaoApiKey,
								)}
								status={getLlmProviderStatus(
									draft.doubaoApiKey,
									presets.doubaoApiKey,
								)}
							/>
							<LabeledField
								label="Kimi API Key"
								value={draft.kimiApiKey}
								onChange={(value) =>
									setDraft((current) => ({ ...current, kimiApiKey: value }))
								}
								placeholder={getLlmProviderStatus(
									draft.kimiApiKey,
									presets.kimiApiKey,
								)}
								status={getLlmProviderStatus(
									draft.kimiApiKey,
									presets.kimiApiKey,
								)}
							/>
							<LabeledField
								label="Zhipu API Key"
								value={draft.zhipuApiKey}
								onChange={(value) =>
									setDraft((current) => ({ ...current, zhipuApiKey: value }))
								}
								placeholder={getLlmProviderStatus(
									draft.zhipuApiKey,
									presets.zhipuApiKey,
								)}
								status={getLlmProviderStatus(
									draft.zhipuApiKey,
									presets.zhipuApiKey,
								)}
							/>
							<LabeledField
								label="DeepSeek API Key"
								value={draft.deepseekApiKey}
								onChange={(value) =>
									setDraft((current) => ({ ...current, deepseekApiKey: value }))
								}
								placeholder={getLlmProviderStatus(
									draft.deepseekApiKey,
									presets.deepseekApiKey,
								)}
								status={getLlmProviderStatus(
									draft.deepseekApiKey,
									presets.deepseekApiKey,
								)}
							/>
							<LabeledField
								label="Ali Qwen API Key"
								value={draft.aliQwenApiKey}
								onChange={(value) =>
									setDraft((current) => ({ ...current, aliQwenApiKey: value }))
								}
								placeholder={getLlmProviderStatus(
									draft.aliQwenApiKey,
									presets.aliQwenApiKey,
								)}
								status={getLlmProviderStatus(
									draft.aliQwenApiKey,
									presets.aliQwenApiKey,
								)}
							/>
						</div>
					</Section>

					<Section
						title="OpenAI"
						description="使用官方 OpenAI Chat Completions 接口。"
					>
						<div className="grid gap-4 md:grid-cols-2">
							<LabeledField
								label="OpenAI API Key"
								value={draft.openaiApiKey}
								onChange={(value) =>
									setDraft((current) => ({ ...current, openaiApiKey: value }))
								}
								placeholder={getLlmProviderStatus(
									draft.openaiApiKey,
									presets.openaiApiKey,
								)}
								status={getLlmProviderStatus(
									draft.openaiApiKey,
									presets.openaiApiKey,
								)}
							/>
							<LabeledField
								label="OpenAI Model"
								value={draft.openaiModel}
								onChange={(value) =>
									setDraft((current) => ({ ...current, openaiModel: value }))
								}
								placeholder="gpt-4.1-mini"
							/>
							<div className="md:col-span-2">
								<LabeledField
									label="OpenAI Base URL"
									value={draft.openaiBaseUrl}
									onChange={(value) =>
										setDraft((current) => ({
											...current,
											openaiBaseUrl: value,
										}))
									}
									placeholder="https://api.openai.com/v1"
								/>
							</div>
						</div>
					</Section>

					<Section
						title="Gemini"
						description="使用 Gemini 的 OpenAI-compatible 接口。"
					>
						<div className="grid gap-4 md:grid-cols-2">
							<LabeledField
								label="Gemini API Key"
								value={draft.geminiApiKey}
								onChange={(value) =>
									setDraft((current) => ({ ...current, geminiApiKey: value }))
								}
								placeholder={getLlmProviderStatus(
									draft.geminiApiKey,
									presets.geminiApiKey,
								)}
								status={getLlmProviderStatus(
									draft.geminiApiKey,
									presets.geminiApiKey,
								)}
								description="如果这里留空，后端会回退到 GEMINI_API_KEY 或 GOOGLE_GENAI_API_KEY。"
							/>
							<LabeledField
								label="Gemini Model"
								value={draft.geminiModel}
								onChange={(value) =>
									setDraft((current) => ({ ...current, geminiModel: value }))
								}
								placeholder="gemini-3-flash-preview"
							/>
							<div className="md:col-span-2">
								<LabeledField
									label="Gemini Base URL"
									value={draft.geminiBaseUrl}
									onChange={(value) =>
										setDraft((current) => ({
											...current,
											geminiBaseUrl: value,
										}))
									}
									placeholder="https://generativelanguage.googleapis.com/v1beta/openai"
								/>
							</div>
						</div>
					</Section>

					<Section
						title="自定义 OpenAI 兼容供应商"
						description="适用于任何遵循 OpenAI Chat Completions 规范的自建或第三方接口。"
					>
						<div className="grid gap-4 md:grid-cols-2">
							<LabeledField
								label="显示名称"
								value={draft.customOpenAiName}
								onChange={(value) =>
									setDraft((current) => ({
										...current,
										customOpenAiName: value,
									}))
								}
								placeholder="自定义 OpenAI 兼容供应商"
							/>
							<LabeledField
								label="模型名"
								value={draft.customOpenAiModel}
								onChange={(value) =>
									setDraft((current) => ({
										...current,
										customOpenAiModel: value,
									}))
								}
								placeholder="your-model-name"
							/>
							<LabeledField
								label="API Key"
								value={draft.customOpenAiApiKey}
								onChange={(value) =>
									setDraft((current) => ({
										...current,
										customOpenAiApiKey: value,
									}))
								}
								placeholder={getLlmProviderStatus(
									draft.customOpenAiApiKey,
									presets.customOpenAiApiKey,
								)}
								status={getLlmProviderStatus(
									draft.customOpenAiApiKey,
									presets.customOpenAiApiKey,
								)}
							/>
							<LabeledField
								label="Base URL"
								value={draft.customOpenAiBaseUrl}
								onChange={(value) =>
									setDraft((current) => ({
										...current,
										customOpenAiBaseUrl: value,
									}))
								}
								placeholder="https://your-endpoint.example/v1"
							/>
						</div>
					</Section>
				</div>

				<div className="flex items-center justify-between gap-3 border-t border-white/10 pt-4">
					<Button
						type="button"
						variant="ghost"
						className="text-slate-200 hover:bg-white/10 hover:text-white"
						onClick={() => {
							setDraft(createDefaultLlmProviderSettings());
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
							className="bg-cyan-300 text-slate-950 hover:bg-cyan-200"
							disabled={!hasChanges}
							onClick={() => {
								updateSettings(draft);
								onOpenChange(false);
								toast.success("大模型 API 配置已保存");
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
