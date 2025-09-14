import {useEffect, useState} from "react"
import ChatList from "./ChatList"
import {Input} from "@/components/ui/input"
import {Button} from "@/components/ui/button"
import {invoke} from "@tauri-apps/api/core";


interface Message {
    role: "user" | "assistant"
    content: string
}

export default function App() {
    useEffect(() => {
        invoke("show_window")
    })
    const [messages, setMessages] = useState<Message[]>([
        {role: "assistant", content: "你好，我是 AI 助手。"},
    ])
    const [input, setInput] = useState("")

    const sendMessage = () => {
        if (!input.trim()) return
        setMessages((prev) => [...prev, {role: "user", content: input}])
        setTimeout(() => {
            setMessages((prev) => [
                ...prev,
                {role: "assistant", content: "这是 AI 的回复。"},
            ])
        }, 800)
        setInput("")
    }

    return (
        <div className="flex flex-col h-screen">
            <ChatList messages={messages}/>

            <div className="border-t p-3 flex gap-2">
                <Input
                    placeholder="输入消息..."
                    value={input}
                    onChange={(e) => setInput(e.target.value)}
                    onKeyDown={(e) => e.key === "Enter" && sendMessage()}
                />
                <Button onClick={sendMessage}>发送</Button>
            </div>
        </div>
    )
}
