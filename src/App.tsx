import { ChatContainer } from "@/components/ChatContainer";
import { Toaster } from "sonner";

function App() {
  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      <ChatContainer />
      <Toaster position="top-right" richColors expand closeButton />
    </div>
  );
}

export default App;
