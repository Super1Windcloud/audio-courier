// env adapter
let nodeCrypto: typeof import("node:crypto") | undefined;
let nodeFs: typeof import("node:fs") | undefined;
let nodeFetch: (
	url: URL | RequestInfo,
	init?: RequestInit,
) => Promise<Response>;
let nodeWebSocket: typeof import("ws")["WebSocket"] | undefined;

const isNode =
	typeof process !== "undefined" &&
	process.versions != null &&
	process.versions.node != null;

if (isNode) {
	nodeCrypto = await import("node:crypto");
	nodeFs = await import("node:fs");
	// @ts-expect-error
	nodeFetch = (await import("node-fetch")).default;
	nodeWebSocket = (await import("ws")).WebSocket;
}

export { isNode, nodeCrypto, nodeFetch, nodeFs, nodeWebSocket };
