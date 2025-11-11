import { useLocation, useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button.tsx";
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
