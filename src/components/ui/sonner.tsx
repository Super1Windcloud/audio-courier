import { useTheme } from "next-themes";
import { Toaster as Sonner } from "sonner";

type ToasterProps = React.ComponentProps<typeof Sonner>;

const Toaster = ({ ...props }: ToasterProps) => {
	const { theme = "system" } = useTheme();

	return (
		<Sonner
			theme={theme as ToasterProps["theme"]}
			className="toaster group"
			toastOptions={{
				classNames: {
					toast:
						"group toast border border-white/10 bg-[linear-gradient(180deg,rgba(28,40,58,0.96)_0%,rgba(18,26,40,0.98)_100%)] text-slate-100 shadow-[0_18px_48px_rgba(2,8,20,0.45)] backdrop-blur-xl",
					success: "border-green-500/50 shadow-[0_0_20px_rgba(34,197,94,0.2)]",
					error: "border-red-500/50 shadow-[0_0_20px_rgba(239,68,68,0.2)]",
					info: "border-purple-500/50 shadow-[0_0_20px_rgba(168,85,247,0.2)]",
					warning:
						"border-purple-500/50 shadow-[0_0_20px_rgba(168,85,247,0.2)]",
					default:
						"border-purple-500/50 shadow-[0_0_20px_rgba(168,85,247,0.2)]",
					description: "text-slate-300",
					actionButton: "bg-cyan-300 text-slate-950 hover:bg-cyan-200",
					cancelButton: "bg-white/10 text-slate-200 hover:bg-white/16",
					title: "text-slate-50",
				},
			}}
			{...props}
		/>
	);
};

export { Toaster };
