import "./App.css";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
	Component,
	lazy,
	Suspense,
	type ReactNode,
	useEffect,
	useRef,
	useState,
} from "react";
import { BrowserRouter, Route, Routes } from "react-router-dom";
import { Toaster, toast } from "sonner";
import { ChatContainer } from "@/components/ChatContainer";
import { logError, logInfo } from "@/lib/logger.ts";
import { registryGlobalShortCuts } from "@/lib/system.ts";
import useAppStateStore from "@/stores";
import type { LicenseStatus } from "@/types/license.ts";

const Conversation = lazy(() =>
	import("@/Conversation.tsx").then((module) => ({
		default: module.Conversation,
	})),
);
const LicenseSignerApp = lazy(() =>
	import("@/components/LicenseSignerApp.tsx").then((module) => ({
		default: module.LicenseSignerApp,
	})),
);

function Home() {
	return (
		<div className="w-full h-screen bg-gradient-to-b from-[#724766] to-[#2C4F71]">
			<ChatContainer />
			<Toaster
				position="top-center"
				richColors
				expand
				closeButton
				duration={5000}
			/>
		</div>
	);
}

class SignerErrorBoundary extends Component<
	{ children: ReactNode },
	{ error: string | null }
> {
	state = { error: null };

	static getDerivedStateFromError(error: Error) {
		return { error: error.message };
	}

	componentDidCatch(error: Error) {
		console.error("license-signer-render-error", error);
		logError("license-signer-render-error", error);
	}

	render() {
		if (this.state.error) {
			return (
				<div className="min-h-screen bg-slate-950 px-6 py-6 text-white">
					<div className="mx-auto flex min-h-[70vh] max-w-3xl items-center justify-center">
						<div className="w-full rounded-3xl border border-red-500/20 bg-red-500/10 p-8 backdrop-blur">
							<p className="text-xs uppercase tracking-[0.35em] text-red-200/70">
								Signer Crash
							</p>
							<h1 className="mt-3 text-3xl font-semibold">签名器渲染失败</h1>
							<p className="mt-4 text-sm text-slate-200">{this.state.error}</p>
							<p className="mt-4 text-xs text-slate-400">
								如果这里出现错误文本，说明不是窗口没打开，而是前端运行时异常。
							</p>
						</div>
					</div>
				</div>
			);
		}

		return this.props.children;
	}
}

function App() {
	const didRun = useRef(false);
	const [windowLabel, setWindowLabel] = useState<string | null>(null);
	const updateLicenseStatus = useAppStateStore(
		(state) => state.updateLicenseStatus,
	);
	const isSignerMode =
		windowLabel === "license-signer" ||
		window.location.hash === "#license-signer" ||
		new URLSearchParams(window.location.search).get("mode") ===
			"license-signer";

	useEffect(() => {
		try {
			setWindowLabel(getCurrentWindow().label);
		} catch (error) {
			console.error("failed-to-read-window-label", error);
			logError("failed-to-read-window-label", error);
		}
	}, []);

	useEffect(() => {
		logInfo(
			`app-mounted label=${windowLabel ?? "unknown"} hash=${window.location.hash || "-"}`,
		);
	}, [windowLabel]);

	useEffect(() => {
		const handleContextMenu = (e: MouseEvent) => {
			e.preventDefault();
		};
		document.addEventListener("contextmenu", handleContextMenu);
		return () => {
			document.removeEventListener("contextmenu", handleContextMenu);
		};
	}, []);

	useEffect(() => {
		if (isSignerMode) {
			return;
		}
		if (didRun.current) return;
		didRun.current = true;
		invoke<LicenseStatus>("get_license_status")
			.then((status) => {
				updateLicenseStatus(status);
			})
			.catch((error) => {
				logError("get_license_status failed", error);
				toast.error(String(error));
			});
		registryGlobalShortCuts().then();
		invoke("show_window").then();
	}, [isSignerMode, updateLicenseStatus]);

	if (isSignerMode) {
		return (
			<SignerErrorBoundary>
				<Suspense fallback={<div className="min-h-screen bg-slate-950" />}>
					<LicenseSignerApp />
					<Toaster
						position="top-center"
						richColors
						expand
						closeButton
						duration={5000}
					/>
				</Suspense>
			</SignerErrorBoundary>
		);
	}

	return (
		<BrowserRouter>
			<Suspense fallback={<Home />}>
				<Routes>
					<Route path="/" element={<Home />} />
					<Route path="/conversation" element={<Conversation />} />
				</Routes>
			</Suspense>
		</BrowserRouter>
	);
}

export default App;
