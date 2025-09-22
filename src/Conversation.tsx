import { Button } from "@/components/ui/button.tsx";
import { useNavigate, useLocation } from "react-router-dom";
export const Conversation = () => {
	const navigate = useNavigate();
	const location = useLocation();
	const { question } = location.state as { question: string };
	return (
		<>
			{question}
			<Button
				onClick={() => {
					navigate("/");
				}}
			>
				Back to Home
			</Button>
		</>
	);
};
