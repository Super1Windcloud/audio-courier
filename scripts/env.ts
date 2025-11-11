// env adapter
let nodeCrypto: typeof import("crypto") | undefined;
let nodeFs: typeof import("fs") | undefined;
let nodeFetch: (
	url: URL | RequestInfo,
	init?: RequestInit,
) => Promise<Response>;
let nodeWebSocket: any;

const isNode =
	typeof process !== "undefined" &&
	process.versions != null &&
	process.versions.node != null;

if (isNode) {
	nodeCrypto = await import("crypto");
	nodeFs = await import("fs");
	// @ts-ignore
	nodeFetch = (await import("node-fetch")).default;
	nodeWebSocket = (await import("ws")).WebSocket;
}

export { isNode, nodeCrypto, nodeFs, nodeFetch, nodeWebSocket };
