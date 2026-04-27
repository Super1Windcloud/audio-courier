import pdfWorkerSrc from "pdfjs-dist/build/pdf.worker.min.mjs?url";

const MAX_RESUME_BYTES = 15 * 1024 * 1024;

type MammothModule = {
	extractRawText(input: { arrayBuffer: ArrayBuffer }): Promise<{
		value: string;
		messages?: Array<{ message: string }>;
	}>;
};

type PdfJsModule = {
	GlobalWorkerOptions: {
		workerSrc: string;
	};
	getDocument(params: { data: Uint8Array }): {
		promise: Promise<PdfDocumentProxy>;
		destroy(): void;
	};
};

type PdfDocumentProxy = {
	numPages: number;
	getPage(pageNumber: number): Promise<PdfPageProxy>;
	destroy(): Promise<void>;
};

type PdfPageProxy = {
	getTextContent(): Promise<{
		items: Array<{ str?: string; hasEOL?: boolean }>;
	}>;
	cleanup(resetStats?: boolean): boolean;
};

export async function extractResumeTextFromFile(file: File): Promise<string> {
	if (file.size === 0) {
		throw new Error("导入文件为空");
	}

	if (file.size > MAX_RESUME_BYTES) {
		throw new Error("文件过大，请导入 15MB 以内的 PDF 或 DOCX 简历");
	}

	const lowerName = file.name.toLowerCase();
	const extracted = lowerName.endsWith(".docx")
		? await extractDocxText(file)
		: lowerName.endsWith(".pdf")
			? await extractPdfText(file)
			: (() => {
					throw new Error("仅支持导入 PDF 或 DOCX 简历");
				})();

	const normalized = normalizeExtractedText(extracted);
	if (!normalized) {
		throw new Error(
			"没有从文件中提取到可用文本，请确认文件不是扫描件或受保护文档",
		);
	}

	return normalized;
}

async function extractDocxText(file: File): Promise<string> {
	const mammothModule = (await import("mammoth")) as MammothModule & {
		default?: MammothModule;
	};
	const mammoth = mammothModule.default ?? mammothModule;
	const result = await mammoth.extractRawText({
		arrayBuffer: await file.arrayBuffer(),
	});

	return result.value;
}

async function extractPdfText(file: File): Promise<string> {
	const pdfjsModule = (await import("pdfjs-dist")) as PdfJsModule;
	pdfjsModule.GlobalWorkerOptions.workerSrc = pdfWorkerSrc;

	const loadingTask = pdfjsModule.getDocument({
		data: new Uint8Array(await file.arrayBuffer()),
	});

	try {
		const pdf = await loadingTask.promise;
		try {
			const pageTexts: string[] = [];
			for (let pageNumber = 1; pageNumber <= pdf.numPages; pageNumber += 1) {
				const page = await pdf.getPage(pageNumber);
				try {
					const textContent = await page.getTextContent();
					const parts: string[] = [];

					for (const item of textContent.items) {
						if (typeof item.str === "string" && item.str.trim()) {
							parts.push(item.str);
						}

						if (item.hasEOL) {
							parts.push("\n");
						}
					}

					const pageText = collapsePdfWhitespace(parts.join(" "));
					if (pageText) {
						pageTexts.push(pageText);
					}
				} finally {
					page.cleanup();
				}
			}

			if (pageTexts.length === 0) {
				throw new Error("未能从 PDF 中提取文本，可能是扫描件或图片型简历");
			}

			return pageTexts.join("\n\n");
		} finally {
			await pdf.destroy();
		}
	} catch (error) {
		const wrappedError = new Error(
			`PDF 文本提取失败: ${error instanceof Error ? error.message : String(error)}`,
		) as Error & { cause?: unknown };
		wrappedError.cause = error;
		throw wrappedError;
	} finally {
		loadingTask.destroy();
	}
}

function collapsePdfWhitespace(input: string) {
	return input
		.replace(/[ \t]+\n/g, "\n")
		.replace(/\n[ \t]+/g, "\n")
		.replace(/[ \t]{2,}/g, " ")
		.replace(/\n{3,}/g, "\n\n")
		.trim();
}

function normalizeExtractedText(input: string) {
	const normalizedLines: string[] = [];
	const normalizedInput = input.replace(/\r\n/g, "\n").replace(/\r/g, "\n");

	for (const rawLine of normalizedInput.split("\n")) {
		const compact = rawLine.split(/\s+/).filter(Boolean).join(" ");
		if (!compact) {
			if (normalizedLines[normalizedLines.length - 1] === "") {
				continue;
			}
			normalizedLines.push("");
			continue;
		}

		if (normalizedLines[normalizedLines.length - 1] === compact) {
			continue;
		}

		normalizedLines.push(compact);
	}

	return normalizedLines.join("\n").trim();
}
