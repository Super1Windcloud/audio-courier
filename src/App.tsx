import "./App.css";
import { getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
	Component,
	lazy,
	type ReactNode,
	Suspense,
	useEffect,
	useEffectEvent,
	useRef,
	useState,
} from "react";
import { BrowserRouter, Route, Routes } from "react-router-dom";
import { toast } from "sonner";
import { ChatContainer } from "@/components/ChatContainer";
import { LicenseSignerApp } from "@/components/LicenseSignerApp.tsx";
import { UpdateDialog } from "@/components/UpdateDialog.tsx";
import { Toaster } from "@/components/ui/sonner.tsx";
import { logError, logInfo } from "@/lib/logger.ts";
import { registryGlobalShortCuts } from "@/lib/system.ts";
import {
	checkForUpdate,
	downloadAndInstallUpdate,
	OPEN_UPDATER_DIALOG_EVENT,
} from "@/lib/updater.ts";
import useAppStateStore from "@/stores";
import type { LicenseStatus } from "@/types/license.ts";

const Conversation = lazy(() =>
	import("@/Conversation.tsx").then((module) => ({
		default: module.Conversation,
	})),
);

function Home() {
	return (
		<div className="h-screen w-full">
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
	const didCheckForUpdates = useRef(false);
	const didRequestWindowShow = useRef(false);
	const [windowLabel, setWindowLabel] = useState<string | null>(null);
	const [availableUpdate, setAvailableUpdate] = useState<Awaited<
		ReturnType<typeof checkForUpdate>
	> | null>(null);
	const [isInstallingUpdate, setIsInstallingUpdate] = useState(false);
	const [updateDialogOpen, setUpdateDialogOpen] = useState(false);
	const [updateProgressTotalBytes, setUpdateProgressTotalBytes] = useState(0);
	const [updateProgressDownloadedBytes, setUpdateProgressDownloadedBytes] =
		useState(0);
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
	}, [isSignerMode, updateLicenseStatus]);

	useEffect(() => {
		if (isSignerMode || didRequestWindowShow.current) {
			return;
		}

		didRequestWindowShow.current = true;

		void invoke("show_window").catch((error) => {
			logError("show_window failed", error);
		});
	}, [isSignerMode]);

	const checkForUpdates = useEffectEvent(
		async (source: "startup" | "manual") => {
			const [currentVersion, update] = await Promise.all([
				getVersion(),
				checkForUpdate(),
			]);
			const remoteVersion = update?.version ?? currentVersion;
			console.log(
				`[updater] ${source} current=${currentVersion} remote=${remoteVersion}`,
			);
			if (!update) {
				console.log("[updater] no update available");
				logInfo(`${source} updater: no update available`);
				if (source === "manual") {
					toast.message("当前已是最新版本");
				}
				setAvailableUpdate(null);
				setUpdateDialogOpen(false);
				return;
			}

			logInfo(`${source} updater: found version ${update.version}`);
			setAvailableUpdate(update);
			setUpdateDialogOpen(true);
			setIsInstallingUpdate(false);
			setUpdateProgressDownloadedBytes(0);
			setUpdateProgressTotalBytes(0);
		},
	);

	useEffect(() => {
		if (isSignerMode || didCheckForUpdates.current) {
			return;
		}

		didCheckForUpdates.current = true;

		void checkForUpdates("startup").catch((error) => {
			console.error("startup updater check failed", error);
			logError("startup updater check failed", error);
		});
	}, [isSignerMode]);

	useEffect(() => {
		if (isSignerMode) {
			return;
		}

		const handleManualUpdateCheck = () => {
			void checkForUpdates("manual").catch((error) => {
				console.error("manual updater check failed", error);
				logError("manual updater check failed", error);
				toast.error(String(error));
			});
		};

		window.addEventListener(OPEN_UPDATER_DIALOG_EVENT, handleManualUpdateCheck);

		return () => {
			window.removeEventListener(
				OPEN_UPDATER_DIALOG_EVENT,
				handleManualUpdateCheck,
			);
		};
	}, [isSignerMode]);

	const handleInstallUpdate = async () => {
		if (!availableUpdate || isInstallingUpdate) {
			return;
		}

		setIsInstallingUpdate(true);
		setUpdateProgressDownloadedBytes(0);
		setUpdateProgressTotalBytes(0);

		try {
			await downloadAndInstallUpdate(availableUpdate, (event) => {
				setUpdateProgressTotalBytes(event.totalBytes);
				setUpdateProgressDownloadedBytes(event.downloadedBytes);
			});
		} catch (error) {
			logError("startup updater install failed", error);
			toast.error(String(error));
			setIsInstallingUpdate(false);
		}
	};

	if (isSignerMode) {
		return (
			<div>
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
			</div>
		);
	}

	return (
		<div>
			<BrowserRouter>
				<Suspense fallback={<Home />}>
					<Routes>
						<Route path="/" element={<Home />} />
						<Route path="/conversation" element={<Conversation />} />
					</Routes>
					<UpdateDialog
						open={updateDialogOpen}
						update={availableUpdate}
						isInstalling={isInstallingUpdate}
						progressTotalBytes={updateProgressTotalBytes}
						progressDownloadedBytes={updateProgressDownloadedBytes}
						onOpenChange={setUpdateDialogOpen}
						onInstall={() => {
							void handleInstallUpdate();
						}}
					/>
				</Suspense>
			</BrowserRouter>
		</div>
	);
}

export default App;
