import type React from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { cn } from "@/lib/utils";

interface MarkdownMessageProps {
	content: string;
	className?: string;
}

export const MarkdownMessage: React.FC<MarkdownMessageProps> = ({
	content,
	className,
}) => {
	return (
		<div
			className={cn(
				"prose prose-invert max-w-none text-sm leading-relaxed break-words",
				"prose-p:my-0 prose-pre:my-2 prose-pre:overflow-x-auto prose-pre:rounded-xl prose-pre:border prose-pre:border-white/10 prose-pre:bg-black/25 prose-pre:p-3",
				"prose-code:rounded prose-code:bg-white/10 prose-code:px-1 prose-code:py-0.5 prose-code:text-[0.92em] prose-code:before:content-none prose-code:after:content-none",
				"prose-ul:my-2 prose-ol:my-2 prose-li:my-0 prose-blockquote:my-2 prose-blockquote:border-white/20 prose-blockquote:text-white/80",
				"prose-headings:mb-2 prose-headings:text-white prose-strong:text-white prose-a:text-sky-300",
				className,
			)}
		>
			<ReactMarkdown remarkPlugins={[remarkGfm]}>{content}</ReactMarkdown>
		</div>
	);
};
