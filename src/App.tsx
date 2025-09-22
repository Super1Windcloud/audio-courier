import "./App.css";
import { ChatContainer } from "@/components/ChatContainer";
import { Toaster } from "sonner";
import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { Conversation } from "@/Conversation.tsx";

function Home() {
	return (
		<div className="h-screen bg-gradient-to-r from-[#4a6e7c] to-[#44497c]">
			<ChatContainer />
			<Toaster position="top-center" richColors expand closeButton />
		</div>
	);
}

function App() {
	useEffect(() => {
		invoke("show_window");
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
