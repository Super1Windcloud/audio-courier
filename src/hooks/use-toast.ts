"use client";

import { toast as sonnerToast } from "sonner";

export function useToast() {
	return {
		toast: (options: { title?: string; description?: string }) => {
			sonnerToast(options.title ?? "", {
				description: options.description,
			});
		},
		success: (message: string, description?: string) => {
			sonnerToast.success(message, { description });
		},
		error: (message: string, description?: string) => {
			sonnerToast.error(message, { description });
		},
		info: (message: string, description?: string) => {
			sonnerToast.message(message, { description });
		},
		warning: (message: string, description?: string) => {
			sonnerToast.warning(message, { description });
		},
	};
}
