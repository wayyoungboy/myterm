import { useState, useEffect, useRef, useCallback } from 'react';
import { Plus, Trash2, Send, MessageSquare, Bot, User } from 'lucide-react';
import type { AiConversation, AiMessage } from '../../types';
import { useAppStore } from '../../stores/appStore';
import {
  getAiConversations,
  createAiConversation,
  deleteAiConversation,
  getAiMessages,
  saveAiMessage,
} from '../../utils/tauri';

const PLACEHOLDER_RESPONSE =
  'AI assistant is not configured yet. Please set up your API key in Settings.';

export default function AiView() {
  const currentView = useAppStore((s) => s.currentView);

  const [conversations, setConversations] = useState<AiConversation[]>([]);
  const [activeConversationId, setActiveConversationId] = useState<string | null>(null);
  const [messages, setMessages] = useState<AiMessage[]>([]);
  const [inputValue, setInputValue] = useState('');
  const [isLoading, setIsLoading] = useState(false);

  const messagesEndRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // ── Load conversations ──────────────────────────────────────────────

  const loadConversations = useCallback(async () => {
    try {
      const list = await getAiConversations();
      setConversations(list);
    } catch (err) {
      console.error('Failed to load AI conversations:', err);
    }
  }, []);

  useEffect(() => {
    if (currentView === 'ai') {
      loadConversations();
    }
  }, [currentView, loadConversations]);

  // ── Load messages when conversation changes ─────────────────────────

  useEffect(() => {
    if (!activeConversationId) {
      setMessages([]);
      return;
    }

    const load = async () => {
      try {
        const msgs = await getAiMessages(activeConversationId);
        setMessages(msgs);
      } catch (err) {
        console.error('Failed to load AI messages:', err);
      }
    };
    load();
  }, [activeConversationId]);

  // ── Auto-scroll ─────────────────────────────────────────────────────

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  // ── Actions ─────────────────────────────────────────────────────────

  const handleNewConversation = async () => {
    try {
      const conv = await createAiConversation();
      await loadConversations();
      setActiveConversationId(conv.id);
    } catch (err) {
      console.error('Failed to create conversation:', err);
    }
  };

  const handleDeleteConversation = async (id: string) => {
    try {
      await deleteAiConversation(id);
      if (activeConversationId === id) {
        setActiveConversationId(null);
        setMessages([]);
      }
      await loadConversations();
    } catch (err) {
      console.error('Failed to delete conversation:', err);
    }
  };

  const handleSendMessage = async () => {
    const text = inputValue.trim();
    if (!text || isLoading) return;

    let conversationId = activeConversationId;

    // Auto-create conversation if none selected
    if (!conversationId) {
      try {
        const conv = await createAiConversation(text.slice(0, 50));
        conversationId = conv.id;
        setActiveConversationId(conv.id);
        await loadConversations();
      } catch (err) {
        console.error('Failed to create conversation:', err);
        return;
      }
    }

    setInputValue('');
    setIsLoading(true);

    try {
      // Save user message
      const userMsg = await saveAiMessage(conversationId, 'user', text);
      setMessages((prev) => [...prev, userMsg]);

      // Simulate a short delay before placeholder response
      await new Promise((resolve) => setTimeout(resolve, 600));

      // Save assistant placeholder message
      const assistantMsg = await saveAiMessage(conversationId, 'assistant', PLACEHOLDER_RESPONSE);
      setMessages((prev) => [...prev, assistantMsg]);
    } catch (err) {
      console.error('Failed to send message:', err);
    } finally {
      setIsLoading(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSendMessage();
    }
  };

  // ── Helpers ─────────────────────────────────────────────────────────

  const formatDate = (dateStr: string | null) => {
    if (!dateStr) return '';
    const d = new Date(dateStr);
    const now = new Date();
    const isToday =
      d.getFullYear() === now.getFullYear() &&
      d.getMonth() === now.getMonth() &&
      d.getDate() === now.getDate();
    if (isToday) {
      return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    }
    return d.toLocaleDateString([], { month: 'short', day: 'numeric' });
  };

  // ── Render ──────────────────────────────────────────────────────────

  return (
    <div className="flex h-full">
      {/* ── Conversation sidebar ──────────────────────────────────── */}
      <div
        className="flex flex-col flex-shrink-0 border-r"
        style={{
          width: 240,
          background: 'var(--bg-secondary)',
          borderColor: 'var(--border)',
        }}
      >
        {/* Header */}
        <div
          className="flex items-center justify-between px-3"
          style={{ height: 44, borderBottom: '1px solid var(--border)' }}
        >
          <span className="text-xs font-medium" style={{ color: 'var(--text-secondary)' }}>
            Conversations
          </span>
          <button
            onClick={handleNewConversation}
            className="flex items-center justify-center rounded"
            style={{ width: 26, height: 26, color: 'var(--text-secondary)' }}
            onMouseEnter={(e) => (e.currentTarget.style.background = 'var(--bg-surface)')}
            onMouseLeave={(e) => (e.currentTarget.style.background = 'transparent')}
            title="New conversation"
          >
            <Plus size={16} />
          </button>
        </div>

        {/* Conversation list */}
        <div className="flex-1 overflow-y-auto py-1">
          {conversations.length === 0 && (
            <div className="px-3 py-6 text-center" style={{ color: 'var(--text-muted)' }}>
              <MessageSquare size={28} className="mx-auto mb-2 opacity-40" />
              <p className="text-xs">No conversations yet</p>
            </div>
          )}

          {conversations.map((conv) => (
            <div
              key={conv.id}
              className="group flex items-center gap-2 px-3 py-2 mx-1 rounded cursor-pointer"
              style={{
                background: activeConversationId === conv.id ? 'var(--bg-surface)' : 'transparent',
              }}
              onClick={() => setActiveConversationId(conv.id)}
              onMouseEnter={(e) => {
                if (activeConversationId !== conv.id)
                  e.currentTarget.style.background = 'var(--bg-hover)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.background =
                  activeConversationId === conv.id ? 'var(--bg-surface)' : 'transparent';
              }}
            >
              <MessageSquare
                size={14}
                className="flex-shrink-0"
                style={{ color: 'var(--text-muted)' }}
              />
              <div className="flex-1 min-w-0">
                <p
                  className="text-xs truncate"
                  style={{ color: 'var(--text-primary)' }}
                >
                  {conv.title || 'New Conversation'}
                </p>
                <p className="text-[10px]" style={{ color: 'var(--text-muted)' }}>
                  {formatDate(conv.created_at)}
                </p>
              </div>
              <button
                className="flex items-center justify-center rounded opacity-0 group-hover:opacity-100 flex-shrink-0"
                style={{ width: 22, height: 22, color: 'var(--text-muted)' }}
                onClick={(e) => {
                  e.stopPropagation();
                  handleDeleteConversation(conv.id);
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.color = 'var(--error)';
                  e.currentTarget.style.background = 'var(--bg-hover)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.color = 'var(--text-muted)';
                  e.currentTarget.style.background = 'transparent';
                }}
                title="Delete conversation"
              >
                <Trash2 size={13} />
              </button>
            </div>
          ))}
        </div>
      </div>

      {/* ── Chat area ────────────────────────────────────────────── */}
      <div className="flex flex-col flex-1 min-w-0" style={{ background: 'var(--bg-primary)' }}>
        {/* Messages */}
        <div className="flex-1 overflow-y-auto px-4 py-4">
          {messages.length === 0 && !isLoading && (
            <div className="flex flex-col items-center justify-center h-full">
              <Bot size={48} className="mb-4" style={{ color: 'var(--text-muted)', opacity: 0.3 }} />
              <p className="text-sm" style={{ color: 'var(--text-muted)' }}>
                {activeConversationId
                  ? 'Start a conversation...'
                  : 'Create or select a conversation to begin'}
              </p>
            </div>
          )}

          {messages.map((msg) => (
            <div
              key={msg.id}
              className={`flex mb-3 ${
                msg.role === 'user' ? 'justify-end' : 'justify-start'
              }`}
            >
              {/* Assistant avatar */}
              {msg.role === 'assistant' && (
                <div
                  className="flex items-center justify-center rounded-full mr-2 flex-shrink-0"
                  style={{
                    width: 28,
                    height: 28,
                    background: 'var(--bg-surface)',
                    color: 'var(--accent)',
                  }}
                >
                  <Bot size={14} />
                </div>
              )}

              <div
                className="rounded-lg px-3 py-2 text-xs leading-relaxed max-w-[70%] whitespace-pre-wrap break-words"
                style={{
                  background:
                    msg.role === 'user' ? 'var(--accent)' : 'var(--bg-surface)',
                  color:
                    msg.role === 'user' ? 'var(--bg-primary)' : 'var(--text-primary)',
                }}
              >
                {msg.content}
              </div>

              {/* User avatar */}
              {msg.role === 'user' && (
                <div
                  className="flex items-center justify-center rounded-full ml-2 flex-shrink-0"
                  style={{
                    width: 28,
                    height: 28,
                    background: 'var(--accent)',
                    color: 'var(--bg-primary)',
                  }}
                >
                  <User size={14} />
                </div>
              )}
            </div>
          ))}

          {/* Loading indicator */}
          {isLoading && (
            <div className="flex justify-start mb-3">
              <div
                className="flex items-center justify-center rounded-full mr-2 flex-shrink-0"
                style={{
                  width: 28,
                  height: 28,
                  background: 'var(--bg-surface)',
                  color: 'var(--accent)',
                }}
              >
                <Bot size={14} />
              </div>
              <div
                className="rounded-lg px-3 py-2 flex items-center gap-1"
                style={{ background: 'var(--bg-surface)' }}
              >
                <span
                  className="inline-block w-1.5 h-1.5 rounded-full animate-bounce"
                  style={{ background: 'var(--text-muted)', animationDelay: '0ms' }}
                />
                <span
                  className="inline-block w-1.5 h-1.5 rounded-full animate-bounce"
                  style={{ background: 'var(--text-muted)', animationDelay: '150ms' }}
                />
                <span
                  className="inline-block w-1.5 h-1.5 rounded-full animate-bounce"
                  style={{ background: 'var(--text-muted)', animationDelay: '300ms' }}
                />
              </div>
            </div>
          )}

          <div ref={messagesEndRef} />
        </div>

        {/* ── Input area ─────────────────────────────────────────── */}
        <div
          className="flex-shrink-0 px-4 py-3"
          style={{ borderTop: '1px solid var(--border)' }}
        >
          <div
            className="flex items-end gap-2 rounded-lg px-3 py-2"
            style={{ background: 'var(--bg-secondary)', border: '1px solid var(--border)' }}
          >
            <textarea
              ref={textareaRef}
              value={inputValue}
              onChange={(e) => setInputValue(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="Type a message... (Shift+Enter for newline)"
              rows={1}
              className="flex-1 resize-none bg-transparent outline-none text-xs leading-relaxed"
              style={{
                color: 'var(--text-primary)',
                maxHeight: 120,
                minHeight: 20,
              }}
              onInput={(e) => {
                const el = e.currentTarget;
                el.style.height = 'auto';
                el.style.height = Math.min(el.scrollHeight, 120) + 'px';
              }}
            />
            <button
              onClick={handleSendMessage}
              disabled={!inputValue.trim() || isLoading}
              className="flex items-center justify-center rounded flex-shrink-0 transition-colors"
              style={{
                width: 30,
                height: 30,
                background: inputValue.trim() && !isLoading ? 'var(--accent)' : 'var(--bg-surface)',
                color:
                  inputValue.trim() && !isLoading
                    ? 'var(--bg-primary)'
                    : 'var(--text-muted)',
                cursor: inputValue.trim() && !isLoading ? 'pointer' : 'default',
              }}
              title="Send message"
            >
              <Send size={14} />
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
