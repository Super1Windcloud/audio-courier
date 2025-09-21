import "./App.css";
import { ChatContainer } from "@/components/ChatContainer";
import { Toaster } from "sonner";
import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

function App() {
	useEffect(() => {
		invoke("show_window");
	}, []);

	return (
		<div className="h-screen bg-gradient-to-r from-[#4a6e7c] to-[#44497c]">
			<ChatContainer />
			<Toaster position="top-center" richColors expand closeButton />
		</div>
	);
}

export default App;
