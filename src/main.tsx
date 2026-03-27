import React from "react";
import ReactDOM from "react-dom/client";
import { initializeAppLogger, logError } from "@/lib/logger.ts";
import App from "./App.tsx";

void initializeAppLogger().catch((error) => {
	console.error("initialize-app-logger-failed", error);
	logError("initialize-app-logger-failed", error);
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
	<React.StrictMode>
		<App />
	</React.StrictMode>,
);
