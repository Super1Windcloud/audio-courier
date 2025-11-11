export function restoreFakePunctuation(input: string): string {
	// 清除多余空格
	let text = input.trim().replace(/\s+/g, "");

	if (text.length === 0) {
		return text;
	}

	// 规则 1：问句结尾词 + 问号
	const questionWords = ["吗", "呢", "是不是", "可不可以", "能不能", "好不好"];
	for (const q of questionWords) {
		if (text.endsWith(q)) {
			text += "？";
			return text;
		}
	}

	// 规则 2：连接词两侧加逗号（避免重复添加）
	const commaWords = [
		"然后",
		"但是",
		"而且",
		"因为",
		"所以",
		"就是",
		"其实",
		"如果",
		"虽然",
		"那么",
		"这样",
	];
	for (const cw of commaWords) {
		// 使用正则替换，确保不重复添加
		const reg = new RegExp(`(?!，)${cw}(?!，)`, "g");
		text = text.replace(reg, `，${cw}，`);
	}

	// 规则 3：句尾语气或停顿加句号
	const periodEndings = ["啊", "吧", "啦", "了", "呢", "噢", "哦", "呀"];
	if (periodEndings.some((end) => text.endsWith(end))) {
		text += "。";
		return text;
	}

	// 规则 4：长度超过阈值自动句号切分
	let result = "";
	let buffer = "";
	let count = 0;

	for (const ch of text) {
		buffer += ch;
		count += 1;
		// 每 15-20 个字加一个句号，但不在 “的” 之后加
		if (count >= 18 && ["我", "他", "你"].includes(ch)) {
			buffer += "。";
			result += buffer;
			buffer = "";
			count = 0;
		}
	}
	result += buffer;

	// 规则 5：末尾补句号
	if (!/[。？！]$/.test(result)) {
		result += "。";
	}

	return result;
}
