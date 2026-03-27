import "./App.css";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef, useState } from "react";
import { BrowserRouter, Route, Routes } from "react-router-dom";
import { toast } from "sonner";
import { Toaster } from "sonner";
import { Conversation } from "@/Conversation.tsx";
import { ChatContainer } from "@/components/ChatContainer";
import { LicenseSignerApp } from "@/components/LicenseSignerApp.tsx";
import { registryGlobalShortCuts } from "@/lib/system.ts";
import useAppStateStore from "@/stores";
import type { LicenseStatus } from "@/types/license.ts";

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

function App() {
	const didRun = useRef(false);
	const [windowLabel] = useState(() => getCurrentWindow().label);
	const updateLicenseStatus = useAppStateStore(
		(state) => state.updateLicenseStatus,
	);
	const isSignerMode = windowLabel === "license-signer";

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
				toast.error(String(error));
			});
		registryGlobalShortCuts().then();
		invoke("show_window").then();
	}, [isSignerMode, updateLicenseStatus]);

	if (isSignerMode) {
		return (
			<>
				<LicenseSignerApp />
				<Toaster position="top-center" richColors expand closeButton duration={5000} />
			</>
		);
	}

	return (
		<BrowserRouter>
			<Routes>
				<Route path="/" element={<Home />} />
				<Route path="/conversation" element={<Conversation />} />
			</Routes>
		</BrowserRouter>
	);
}

export default App;
