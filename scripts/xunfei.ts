import { isNode, nodeCrypto, nodeFs, nodeWebSocket } from "./env";

type FixedParams = {
	audio_encode: string;
	lang: string;
	samplerate: string;
};

const FIXED_PARAMS: FixedParams = {
	audio_encode: "pcm_s16le",
	lang: "autodialect",
	samplerate: "16000",
};

const AUDIO_FRAME_SIZE = 1280; // 每帧字节数（16k,16bit,40ms）
const FRAME_INTERVAL_MS = 40;

function utf8Encode(str: string): Uint8Array {
	return new TextEncoder().encode(str);
}
function arrayBufferToBase64(buffer: ArrayBuffer): string {
	const bytes = new Uint8Array(buffer);
	let binary = "";
	const chunkSize = 0x8000;
	for (let i = 0; i < bytes.length; i += chunkSize) {
		binary += String.fromCharCode.apply(
			null,
			Array.from(bytes.slice(i, i + chunkSize)),
		);
	}
	return btoa(binary);
}

function randUuidHex() {
	// 简单生成 32 字符 hex uuid
	// 浏览器环境用 crypto.randomUUID()（若有），否则 fallback
	if ("randomUUID" in crypto) return crypto.randomUUID().replace(/-/g, "");
	const arr = new Uint8Array(16);
	// @ts-ignore
	crypto.getRandomValues(arr);
	return Array.prototype.map
		.call(arr, (x: number) => ("00" + x.toString(16)).slice(-2))
		.join("");
}
function formatUtcPlus8(): string {
	// 生成 yyyy-MM-dd'T'HH:mm:ss+0800
	// 采用当前 UTC 时间 +8小时
	const now = new Date(Date.now() + 8 * 3600 * 1000);
	const yyyy = now.getUTCFullYear();
	const mm = String(now.getUTCMonth() + 1).padStart(2, "0");
	const dd = String(now.getUTCDate()).padStart(2, "0");
	const hh = String(now.getUTCHours()).padStart(2, "0");
	const min = String(now.getUTCMinutes()).padStart(2, "0");
	const ss = String(now.getUTCSeconds()).padStart(2, "0");
	return `${yyyy}-${mm}-${dd}T${hh}:${min}:${ss}+0800`;
}
// 定义单个词（Word）的结构
interface CandidateWord {
	sc: number;
	w: string; // 我们要提取的文本
	wp: string;
	rl: string;
	wb: number;
	wc: number;
	we: number;
}

// 定义包含词汇列表（cw）的对象结构
interface WordSegment {
	cw: CandidateWord[];
	wb: number;
	we: number;
}

// 完整的 JSON 数组结构
type ASRResult = WordSegment[];

class RTASRClientTS {
	appId: string;
	accessKeyId: string;
	accessKeySecret: string;
	baseWsUrl: string =
		"wss://office-api-ast-dx.iflyaisol.com/ast/communicate/v1";

	ws: WebSocket | null = null;
	isConnected = false;
	sessionId: string | null = null;
	isSendingAudio = false;
	audioFileSize = 0;

	// UI callback
	logFn: (s: string) => void;

	constructor(
		appId: string,
		accessKeyId: string,
		accessKeySecret: string,
		logFn: (s: string) => void,
	) {
		this.appId = appId;
		this.accessKeyId = accessKeyId;
		this.accessKeySecret = accessKeySecret;
		this.logFn = logFn;
	}

	private log(msg: string) {
		console.log(msg);
		try {
			this.logFn(msg);
		} catch (error) {
			// Ignore errors thrown by the logging callback.
			void error;
		}
	}

	private urlEncodeSortedParams(params: Record<string, string>) {
		const entries = Object.entries(params)
			.filter(([_k, v]) => v != null && String(v).trim() !== "")
			.sort((a, b) => a[0].localeCompare(b[0], undefined, { numeric: false }));
		return entries
			.map(([k, v]) => `${encodeURIComponent(k)}=${encodeURIComponent(v)}`)
			.join("&");
	}

	private async hmacSha1Base64(keyStr: string, messageStr: string) {
		if (isNode) {
			// Node: 使用 crypto.createHmac
			const h = nodeCrypto!.createHmac("sha1", keyStr);
			h.update(messageStr);
			return h.digest("base64");
		} else {
			// 浏览器: 使用 subtleCrypto
			const keyData = utf8Encode(keyStr);
			const cryptoKey = await crypto.subtle.importKey(
				"raw",
				keyData,
				{ name: "HMAC", hash: { name: "SHA-1" } },
				false,
				["sign"],
			);
			const signature = await crypto.subtle.sign(
				"HMAC",
				cryptoKey,
				utf8Encode(messageStr),
			);
			return arrayBufferToBase64(signature);
		}
	}

	async generateAuthParams(): Promise<Record<string, string>> {
		const authParams: Record<string, string> = {
			accessKeyId: this.accessKeyId,
			appId: this.appId,
			uuid: randUuidHex(),
			utc: formatUtcPlus8(),
			...FIXED_PARAMS,
		};

		// 按字典序排序 + URL encode
		const baseStr = this.urlEncodeSortedParams(authParams);
		this.log(`【鉴权基础字符串】${baseStr}`);

		const signature = await this.hmacSha1Base64(this.accessKeySecret, baseStr);
		authParams["signature"] = signature;
		return authParams;
	}

	async connect(): Promise<boolean> {
		try {
			const auth = await this.generateAuthParams();
			const paramsStr = Object.entries(auth)
				.map(([k, v]) => `${encodeURIComponent(k)}=${encodeURIComponent(v)}`)
				.join("&");
			const fullWsUrl = `${this.baseWsUrl}?${paramsStr}`;
			this.log(`【连接信息】完整URL：${fullWsUrl}`);

			this.ws = isNode
				? new nodeWebSocket!(fullWsUrl)
				: new WebSocket(fullWsUrl);

			// 二进制类型为 ArrayBuffer
			this.ws!.binaryType = "arraybuffer";

			return await new Promise<boolean>((resolve) => {
				if (!this.ws) return resolve(false);

				const onOpen = () => {
					this.isConnected = true;
					this.log("【连接成功】WebSocket 握手完成，等待服务端就绪...");
					// 注册消息回调
					this.ws!.onmessage = (ev) => this.onMessage(ev);
					this.ws!.onclose = (ev) => {
						this.isConnected = false;
						this.log(`【连接关闭】code=${ev.code} reason=${ev.reason}`);
					};
					this.ws!.onerror = (ev) => {
						this.log("【连接错误】" + JSON.stringify(ev));
					};
					resolve(true);
				};
				const onError = (ev: Event) => {
					this.log("【连接失败】WebSocket 错误：" + JSON.stringify(ev));
					resolve(false);
				};

				this.ws!.addEventListener("open", onOpen, { once: true });
				this.ws!.addEventListener("error", onError, { once: true });

				// 超时保护（15s）
				setTimeout(() => {
					if (!this.isConnected) {
						this.log("【连接超时】未在 15s 内建立连接");
						try {
							this.ws?.close();
						} catch (error) {
							void error;
						}
						resolve(false);
					}
				}, 15000);
			});
		} catch (error) {
			const message =
				error instanceof Error
					? error.message
					: typeof error === "string"
						? error
						: JSON.stringify(error);
			this.log(`【连接异常】${message}`);
			return false;
		}
	}

	private onMessage(ev: MessageEvent) {
		if (typeof ev.data === "string") {
			try {
				const json = JSON.parse(ev.data);
				const text = json["data"]["cn"]["st"]["rt"][0]["ws"] as ASRResult;
				this.log("【接收消息】" + this.extractTextFromASR(text));
				if (json?.msg_type === "action" && json?.data?.sessionId) {
					this.sessionId = json.data.sessionId;
					this.log(`【会话更新】sessionId=${this.sessionId}`);
				}
			} catch (_e) {
				this.log("【接收异常】非 JSON 文本消息：" + String(ev.data));
			}
		} else if (ev.data instanceof ArrayBuffer) {
			this.log(`【接收二进制】长度：${ev.data.byteLength} 字节（忽略）`);
		} else {
			this.log("【接收】未知消息类型");
		}
	}

	public extractTextFromASR(data: ASRResult): string {
		let fullText = "";

		// 1. 遍历顶层数组（WordSegment）
		for (const segment of data) {
			// 2. 遍历每个 WordSegment 中的 cw 数组
			for (const word of segment.cw) {
				// 3. 拼接每个词汇的文本
				fullText += word.w;
			}
		}

		return fullText;
	}

	async sendAudioFile(file: File | Blob | string | Buffer): Promise<boolean> {
		if (!this.isConnected || !this.ws) {
			this.log("【发送失败】WebSocket 未连接");
			return false;
		}
		if (this.isSendingAudio) {
			this.log("【发送失败】已有发送任务在执行");
			return false;
		}
		this.isSendingAudio = true;

		try {
			let arrayBuffer: ArrayBuffer;
			if (isNode) {
				let buffer: Buffer;
				if (typeof file === "string") buffer = nodeFs!.readFileSync(file);
				else if (Buffer.isBuffer(file)) buffer = file;
				else throw new Error("Node 环境需传入路径或 Buffer");
				arrayBuffer = buffer.buffer.slice(
					buffer.byteOffset,
					buffer.byteOffset + buffer.byteLength,
				);
			} else {
				// 浏览器
				if (!(file instanceof Blob)) throw new Error("浏览器需传入 Blob/File");
				arrayBuffer = await file.arrayBuffer();
			}

			this.audioFileSize = arrayBuffer.byteLength;
			// 计算总帧数
			let totalFrames = Math.floor(this.audioFileSize / AUDIO_FRAME_SIZE);
			const remaining = this.audioFileSize % AUDIO_FRAME_SIZE;
			if (remaining > 0) totalFrames += 1;
			const estimatedDuration = (totalFrames * FRAME_INTERVAL_MS) / 1000;
			this.log(
				`【发送配置】音频文件大小：${this.audioFileSize} 字节 | 总帧数：${totalFrames} | 预估时长：${estimatedDuration.toFixed(
					1,
				)} 秒`,
			);

			const view = new Uint8Array(arrayBuffer);
			let frameIndex = 0;
			let startTime: number | null = null; // performance.now() 基准（ms）

			while (frameIndex * AUDIO_FRAME_SIZE < this.audioFileSize) {
				const startByte = frameIndex * AUDIO_FRAME_SIZE;
				const endByte = Math.min(
					startByte + AUDIO_FRAME_SIZE,
					this.audioFileSize,
				);
				const chunk = view.slice(startByte, endByte);

				if (startTime === null) {
					startTime = performance.now();
					this.log(`【发送开始】起始时间：${startTime.toFixed(1)} ms（基准）`);
				}

				const expectedSendTime = startTime + frameIndex * FRAME_INTERVAL_MS;
				const now = performance.now();
				const diff = expectedSendTime - now;

				if (diff > 0.1) {
					// 等待到理论发送时间（避免过度休眠）
					await new Promise((r) => setTimeout(r, diff));
					// 每 10 帧打印一次节奏日志
					if (frameIndex % 10 === 0) {
						const actual = performance.now();
						this.log(
							`【节奏控制】帧${frameIndex} | 理论：${expectedSendTime.toFixed(
								1,
							)} | 实际：${actual.toFixed(1)} | 误差：${(actual - expectedSendTime).toFixed(1)} ms`,
						);
					}
				}

				// 发送二进制
				try {
					this.ws.send(chunk.buffer);
				} catch (e) {
					this.log("【发送异常】WebSocket 发送失败：" + String(e));
					this.close();
					return false;
				}

				frameIndex += 1;
			}

			// 发送结束标记
			const endMsg: { end: true; sessionId?: string } = { end: true };
			if (this.sessionId) endMsg.sessionId = this.sessionId;
			const endStr = JSON.stringify(endMsg);
			this.ws.send(endStr);
			this.log(`【发送结束】已发送结束标记：${endStr}`);
			return true;
		} catch (error) {
			const message =
				error instanceof Error
					? error.message
					: typeof error === "string"
						? error
						: JSON.stringify(error);
			this.log("【发送异常】" + message);
			this.close();
			return false;
		} finally {
			this.isSendingAudio = false;
		}
	}

	close() {
		if (this.ws && this.isConnected) {
			try {
				this.ws.close(1000, "客户端正常关闭");
			} catch (e) {
				this.log("【关闭异常】" + String(e));
			}
		} else {
			this.log("【连接关闭】WebSocket 未连接或已断开");
		}
		this.isConnected = false;
		this.ws = null;
	}
}

async function testAudioStreamNode() {
	const appID = "";
	const apiKey = "";
	const apiSecret = "";

	const client = new RTASRClientTS(appID, apiKey, apiSecret, console.log);
	const ok = await client.connect();
	if (!ok) return;

	console.log("开始发送音频...");
	// 只允许单声道, 16比特位, 16000采样率,
	await client.sendAudioFile("../src-tauri/recorded_i16_mono_resample.wav");
	setTimeout(() => client.close(), 1000);
}

testAudioStreamNode();
