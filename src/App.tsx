import "./App.css";
import { ChatContainer } from "@/components/ChatContainer";
import { Toaster } from "sonner";
import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { Conversation } from "@/Conversation.tsx";
import { registryGlobalShortCuts } from "@/lib/system.ts";

function Home() {
  return (
    <div className="w-full h-screen bg-gradient-to-b from-[#724766] to-[#2C4F71]">
      <ChatContainer />
      <Toaster position="top-center" richColors expand closeButton />
    </div>
  );
}

function App() {
  useEffect(() => {
    registryGlobalShortCuts();
    invoke("show_window");
  }, []);

  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Home />} />
        <Route path="/conversation" element={<Conversation />} />
      </Routes>
    </BrowserRouter>
  );
}

export default App;
