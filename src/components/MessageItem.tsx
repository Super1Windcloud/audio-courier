import { useState, useEffect, useCallback } from 'react'
import MarkdownIt from 'markdown-it'
import mdKatex from 'markdown-it-katex'
import mdHighlight from 'markdown-it-highlightjs'
import IconRefresh from './icons/Refresh'
import type { ChatMessage } from '@/types'


interface Props {
    role: ChatMessage['role']
    message: string | (() => string)
    showRetry?: () => boolean
    onRetry?: () => void
}

export default function ChatMessageItem({ role, message, showRetry, onRetry }: Props) {
    const roleClass: Record<string, string> = {
        system: 'bg-gradient-to-r from-gray-300 via-gray-200 to-gray-300',
        user: 'bg-gradient-to-r from-purple-400 to-yellow-400',
        assistant: 'bg-gradient-to-r from-yellow-200 via-green-200 to-green-300',
    }

    const [copied, setCopied] = useState(false)

    const copy = useCallback(async (text: string) => {
        try {
            await navigator.clipboard.writeText(text)
            setCopied(true)
            setTimeout(() => setCopied(false), 1000)
        } catch (err) {
            console.error('Copy failed', err)
        }
    }, [])

    // 事件监听（复制按钮）
    useEffect(() => {
        const handleClick = (e: Event) => {
            const el = e.target as HTMLElement
            let code: string | null = null

            if (el.matches('div > div.copy-btn')) {
                code = decodeURIComponent(el.dataset.code!)
                copy(code)
            }
            if (el.matches('div > div.copy-btn > svg')) {
                code = decodeURIComponent(el.parentElement?.dataset.code!)
                copy(code)
            }
        }

        document.addEventListener('click', handleClick)
        return () => {
            document.removeEventListener('click', handleClick)
        }
    }, [copy])

    const renderHtml = useCallback(() => {
        const md = MarkdownIt({
            linkify: true,
            breaks: true,
        }).use(mdKatex).use(mdHighlight)

        const fence = md.renderer.rules.fence!
        md.renderer.rules.fence = (...args) => {
            const [tokens, idx] = args
            const token = tokens[idx]
            const rawCode = fence(...args)

            return `<div relative>
        <div data-code=${encodeURIComponent(token.content)} class="copy-btn gpt-copy-btn group">
          <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 32 32"><path fill="currentColor" d="M28 10v18H10V10h18m0-2H10a2 2 0 0 0-2 2v18a2 2 0 0 0 2-2V10a2 2 0 0 0 2-2V10a2 2 0 0 0-2-2Z" /><path fill="currentColor" d="M4 18H2V4a2 2 0 0 1 2-2h14v2H4Z" /></svg>
          <div class="group-hover:op-100 gpt-copy-tips">
            ${copied ? 'Copied' : 'Copy'}
          </div>
        </div>
        ${rawCode}
      </div>`
        }

        if (typeof message === 'function') {
            return md.render(message())
        } else if (typeof message === 'string') {
            return md.render(message)
        }
        return ''
    }, [message, copied])

    return (
        <div className="py-2 -mx-4 px-4 transition-colors md:hover:bg-slate/3">
            <div className={`flex gap-3 rounded-lg ${role === 'user' ? 'opacity-75' : ''}`}>
                <div className={`shrink-0 w-7 h-7 mt-4 rounded-full opacity-80 ${roleClass[role]}`} />
                <div
                    className="message prose break-words overflow-hidden"
                    dangerouslySetInnerHTML={{ __html: renderHtml() }}
                />
            </div>
            {showRetry?.() && onRetry && (
                <div className="fie px-3 mb-2">
                    <div onClick={onRetry} className="gpt-retry-btn">
                        <IconRefresh />
                        <span>Regenerate</span>
                    </div>
                </div>
            )}
        </div>
    )
}
