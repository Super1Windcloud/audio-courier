import { openUrl } from "@tauri-apps/plugin-opener";
import { ExternalLink } from "lucide-react";
import { toast } from "sonner";
import { Input } from "@/components/ui/input.tsx";
import type { ProviderOfficialLink } from "@/lib/providerOfficialLinks.ts";

interface ProviderConfigFieldProps {
	label: string;
	value: string;
	onChange: (value: string) => void;
	placeholder?: string;
	description?: string;
	status?: string;
	officialLink?: ProviderOfficialLink;
}

export function ProviderConfigField({
	label,
	value,
	onChange,
	placeholder,
	description,
	status,
	officialLink,
}: ProviderConfigFieldProps) {
	const handleOpenOfficialLink = () => {
		if (!officialLink) {
			return;
		}

		void openUrl(officialLink.url).catch((error) => {
			console.error(
				"failed to open provider official url via tauri opener",
				error,
			);

			if (typeof window !== "undefined") {
				window.open(officialLink.url, "_blank", "noopener,noreferrer");
				return;
			}

			toast.error("打开官网失败");
		});
	};

	return (
		<div className="grid gap-2">
			<div className="flex items-center justify-between gap-3">
				<div className="flex flex-wrap items-center gap-2">
					<span className="text-sm font-medium text-white">{label}</span>
					{officialLink ? (
						<button
							type="button"
							onClick={handleOpenOfficialLink}
							className="inline-flex items-center gap-1 rounded-full border border-cyan-400/20 bg-cyan-400/10 px-2.5 py-1 text-[11px] text-cyan-100 transition-colors hover:border-cyan-300/40 hover:bg-cyan-300/15 hover:text-white"
						>
							<ExternalLink className="size-3" />
							{officialLink.label ?? "官网"}
						</button>
					) : null}
				</div>
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
