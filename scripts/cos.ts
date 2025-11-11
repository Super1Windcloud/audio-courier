// 测试：两个 1024 维随机向量
const dim = 1024;
const vecA = Array.from({ length: dim }, () => Math.random() - 0.5);
const vecB = Array.from({ length: dim }, () => Math.random() - 0.5);

function getVectorCosineSimilarty(a: number[], b: number[]) {
	let dotResult = 0;
	for (let i = 0; i < a.length; ++i) {
		dotResult += a[i] * b[i];
	}
	let ANorm = 0;
	let BNorm = 0;
	for (let i = 0; i < a.length; ++i) {
		ANorm += a[i] * a[i];
		BNorm += b[i] * b[i];
	}

	const result = dotResult / (Math.sqrt(ANorm) * Math.sqrt(BNorm));
	console.log(result);
}

function cosineSimilarity(a: number[], b: number[]): void {
	if (a.length !== b.length) {
		throw new Error("向量维度不一致");
	}

	// 点积
	const dot = a.reduce((sum, ai, i) => sum + ai * b[i], 0);

	// 模长
	const normA = Math.sqrt(a.reduce((sum, ai) => sum + ai * ai, 0));
	const normB = Math.sqrt(b.reduce((sum, bi) => sum + bi * bi, 0));

	const result = dot / (normA * normB);
	console.log(result);
}

getVectorCosineSimilarty(vecA, vecB);
cosineSimilarity(vecA, vecB);
