export interface AudioChannelOption {
	value: string;
	name: string;
	kind: "output" | "input";
	isDefault: boolean;
}

function parseLegacyAudioChannel(target: string) {
	const normalized = target.trim();

	if (!normalized) {
		return null;
	}

	if (normalized.endsWith(" [输出] (默认)")) {
		return {
			name: normalized.slice(0, -"[输出] (默认)".length).trim(),
			kind: "output" as const,
			isDefault: true,
		};
	}

	if (normalized.endsWith(" [输出]")) {
		return {
			name: normalized.slice(0, -"[输出]".length).trim(),
			kind: "output" as const,
			isDefault: false,
		};
	}

	if (normalized.endsWith(" [输入]")) {
		return {
			name: normalized.slice(0, -"[输入]".length).trim(),
			kind: "input" as const,
			isDefault: false,
		};
	}

	return null;
}

export function isOutputAudioChannel(target: string) {
	return target.startsWith("output:") || target.includes("输出");
}

export function findAudioChannelValue(
	options: AudioChannelOption[],
	currentValue: string,
) {
	const exactMatch = options.find((option) => option.value === currentValue);

	if (exactMatch) {
		return exactMatch.value;
	}

	const legacyMatch = parseLegacyAudioChannel(currentValue);

	if (!legacyMatch) {
		return null;
	}

	return (
		options.find(
			(option) =>
				option.name === legacyMatch.name &&
				option.kind === legacyMatch.kind &&
				option.isDefault === legacyMatch.isDefault,
		)?.value ?? null
	);
}
