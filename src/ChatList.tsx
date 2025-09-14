// src/components/ChatList.tsx
import {useEffect, useRef} from "react"
import {Card, CardContent} from "@/components/ui/card"

interface Message {
    role: "user" | "assistant"
    content: string
}

interface ChatListProps {
    messages: Message[]
}

export default function ChatList({messages}: ChatListProps) {
    const bottomRef = useRef<HTMLDivElement | null>(null)

    // 滚动到底部
    useEffect(() => {
        bottomRef.current?.scrollIntoView({behavior: "smooth"})
    }, [messages])

    return (
        <div className="flex-1 overflow-y-auto p-4 space-y-3 bg-zinc-50">
            {messages.map((msg, idx) => (
                <div
                    key={idx}
                    className={`flex ${
                        msg.role === "user" ? "justify-end" : "justify-start"
                    }`}
                >
                    <Card
                        className={`max-w-xs shadow-sm ${
                            msg.role === "user"
                                ? "bg-zinc-800 text-white"
                                : "bg-white border border-zinc-200"
                        }`}
                    >
                        <CardContent className="p-3 text-sm">
                            {msg.content}
                        </CardContent>
                    </Card>
                </div>
            ))}
            <div ref={bottomRef}/>
        </div>
    )
}
