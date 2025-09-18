import MessageItem from "@/components/MessageItem.tsx";
import { invoke } from "@tauri-apps/api/core";
import { useEffect } from "react";
export default function App() {
	useEffect(() => {
		(async () => {
			await invoke("show_window");
		})();
	}, []);

	return (
		<MessageItem
			username={"src-tauri"}
			message={"jifoasejfoisejfoaisefjiaseof"}
			time={"24.00"}
		></MessageItem>
	);
}
