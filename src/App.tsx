import "./App.css";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef } from "react";
import { BrowserRouter, Route, Routes } from "react-router-dom";
import { Toaster } from "sonner";
import { Conversation } from "@/Conversation.tsx";
import { ChatContainer } from "@/components/ChatContainer";
import { registryGlobalShortCuts } from "@/lib/system.ts";

function Home() {
	return (
		<div className="w-full h-screen bg-gradient-to-b from-[#724766] to-[#2C4F71]">
			<ChatContainer />
			<Toaster position="top-center" richColors expand closeButton  duration={5000}/>
		</div>
	);
}

function App() {
	const didRun = useRef(false);

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
		if (didRun.current) return;
		didRun.current = true;
		registryGlobalShortCuts().then();
		invoke("show_window").then();
	}, []);

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
