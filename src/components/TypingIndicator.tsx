import type React from "react";
import { useEffect, useState } from "react";

export const TypingIndicator: React.FC = () => {
	const [elapsedSeconds, setElapsedSeconds] = useState(0);

	useEffect(() => {
		const startedAt = Date.now();
		const timer = window.setInterval(() => {
			setElapsedSeconds(Math.floor((Date.now() - startedAt) / 1000));
		}, 1000);

		return () => {
			window.clearInterval(timer);
		};
	}, []);

	const minutes = Math.floor(elapsedSeconds / 60)
		.toString()
		.padStart(2, "0");
	const seconds = (elapsedSeconds % 60).toString().padStart(2, "0");

	return (
		<div className="flex items-end space-x-2 animate-in slide-in-from-bottom-2 duration-300">
			<div className="rounded-2xl rounded-br-md rounded-bl-md border border-blue-200/10 bg-blue-500/10 px-4 py-2 text-white shadow-lg shadow-black/15 backdrop-blur-md">
				<div className="flex items-center gap-3">
					<div className="flex space-x-1">
						<div
							className="h-2 w-2 animate-bounce rounded-full bg-muted-foreground"
							style={{ animationDelay: "0ms" }}
						/>
						<div
							className="h-2 w-2 animate-bounce rounded-full bg-muted-foreground"
							style={{ animationDelay: "150ms" }}
						/>
						<div
							className="h-2 w-2 animate-bounce rounded-full bg-muted-foreground"
							style={{ animationDelay: "300ms" }}
						/>
					</div>
					<div
						className="min-w-[4.25rem] text-right font-mono text-xs tabular-nums text-blue-100/90"
						aria-label={`等待 AI 输出 ${minutes}:${seconds}`}
						role="timer"
					>
						{minutes}:{seconds}
					</div>
				</div>
			</div>
		</div>
	);
};
