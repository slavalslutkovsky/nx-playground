import { createSignal, createEffect, createMemo, For, Show, onCleanup } from "solid-js";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/card";
import { Button } from "~/components/ui/button";
import { Input } from "~/components/ui/input";
import { streamChatMessage, createSession, fetchSessions } from "~/lib/api-client";
import { useAuth } from "@nx-playground/auth-solid";
import type { ChatSession, ChatContext, CloudProvider } from "~/types";

interface Message {
  id: string;
  role: "user" | "assistant" | "tool";
  content: string;
  toolCalls?: Array<{
    name: string;
    arguments: string;
    result?: string;
  }>;
  timestamp: Date;
}


const EXAMPLE_PROMPTS = [
  "Compare PostgreSQL pricing across AWS, Azure, and GCP",
  "What's the cheapest way to run a small Redis cluster?",
  "Help me optimize my cloud infrastructure costs",
  "Calculate TCO for running Kafka on Kubernetes vs managed services",
];

export default function FinopsChat() {
  const auth = useAuth();
  // user_id is optional - null for unauthenticated users (guest mode)
  const userId = createMemo(() => auth.user()?.id);

  const [messages, setMessages] = createSignal<Message[]>([]);
  const [input, setInput] = createSignal("");
  const [isLoading, setIsLoading] = createSignal(false);
  const [session, setSession] = createSignal<ChatSession | null>(null);
  const [sessions, setSessions] = createSignal<ChatSession[]>([]);
  const [showSidebar, setShowSidebar] = createSignal(true);
  const [currentToolCall, setCurrentToolCall] = createSignal<string | null>(null);
  const [context, setContext] = createSignal<ChatContext>({
    preferred_providers: [],
    budget_monthly: null,
    regions: [],
    compliance_requirements: [],
    cloud_account_ids: [],
  });

  let messagesEndRef: HTMLDivElement | undefined;
  let cleanupFn: (() => void) | null = null;

  // Load sessions on mount
  createEffect(() => {
    loadSessions();
  });

  // Scroll to bottom when messages change
  createEffect(() => {
    messages(); // Track dependency
    messagesEndRef?.scrollIntoView({ behavior: "smooth" });
  });

  // Cleanup on unmount
  onCleanup(() => {
    cleanupFn?.();
  });

  async function loadSessions() {
    try {
      const data = await fetchSessions(userId());
      setSessions(data);
    } catch (error) {
      console.error("Failed to load sessions:", error);
    }
  }

  async function startNewSession() {
    try {
      const newSession = await createSession({
        user_id: userId(),
        context: context(),
      });
      setSession(newSession);
      setMessages([]);
      await loadSessions();
    } catch (error) {
      console.error("Failed to create session:", error);
    }
  }

  function selectSession(selectedSession: ChatSession) {
    setSession(selectedSession);
    setMessages([]);
    // TODO: Load messages for this session
  }

  function handleExampleClick(prompt: string) {
    setInput(prompt);
  }

  async function sendMessage() {
    const messageText = input().trim();
    if (!messageText || isLoading()) return;

    // Create session if needed
    let currentSession = session();
    if (!currentSession) {
      try {
        currentSession = await createSession({
          user_id: userId(),
          title: messageText.slice(0, 50),
          context: context(),
        });
        setSession(currentSession);
        await loadSessions();
      } catch (error) {
        console.error("Failed to create session:", error);
        return;
      }
    }

    // Add user message
    const userMessage: Message = {
      id: crypto.randomUUID(),
      role: "user",
      content: messageText,
      timestamp: new Date(),
    };
    setMessages((prev) => [...prev, userMessage]);
    setInput("");
    setIsLoading(true);

    // Start assistant message placeholder
    const assistantMessageId = crypto.randomUUID();
    const assistantMessage: Message = {
      id: assistantMessageId,
      role: "assistant",
      content: "",
      toolCalls: [],
      timestamp: new Date(),
    };
    setMessages((prev) => [...prev, assistantMessage]);

    // Stream response
    cleanupFn = streamChatMessage(
      {
        session_id: currentSession.id,
        user_id: userId(),
        message: messageText,
        context: context(),
      },
      (event, data) => {
        switch (event) {
          case "text":
            setMessages((prev) =>
              prev.map((m) =>
                m.id === assistantMessageId
                  ? { ...m, content: m.content + data }
                  : m
              )
            );
            break;
          case "tool_call":
            try {
              const toolData = JSON.parse(data);
              setCurrentToolCall(toolData.name);
              setMessages((prev) =>
                prev.map((m) =>
                  m.id === assistantMessageId
                    ? {
                        ...m,
                        toolCalls: [
                          ...(m.toolCalls || []),
                          { name: toolData.name, arguments: toolData.arguments },
                        ],
                      }
                    : m
                )
              );
            } catch (e) {
              console.error("Failed to parse tool call:", e);
            }
            break;
          case "tool_result":
            try {
              const resultData = JSON.parse(data);
              setCurrentToolCall(null);
              setMessages((prev) =>
                prev.map((m) =>
                  m.id === assistantMessageId
                    ? {
                        ...m,
                        toolCalls: m.toolCalls?.map((tc) =>
                          tc.name === resultData.name
                            ? { ...tc, result: resultData.result }
                            : tc
                        ),
                      }
                    : m
                )
              );
            } catch (e) {
              console.error("Failed to parse tool result:", e);
            }
            break;
          case "done":
            setIsLoading(false);
            cleanupFn = null;
            break;
          case "error":
            setMessages((prev) =>
              prev.map((m) =>
                m.id === assistantMessageId
                  ? { ...m, content: `Error: ${data}` }
                  : m
              )
            );
            setIsLoading(false);
            cleanupFn = null;
            break;
        }
      },
      (error) => {
        console.error("Stream error:", error);
        setMessages((prev) =>
          prev.map((m) =>
            m.id === assistantMessageId
              ? { ...m, content: "Connection error. Please try again." }
              : m
          )
        );
        setIsLoading(false);
        cleanupFn = null;
      },
      () => {
        setIsLoading(false);
        cleanupFn = null;
      }
    );
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  }

  function toggleProvider(provider: CloudProvider) {
    setContext((prev) => ({
      ...prev,
      preferred_providers: prev.preferred_providers.includes(provider)
        ? prev.preferred_providers.filter((p) => p !== provider)
        : [...prev.preferred_providers, provider],
    }));
  }

  return (
    <div class="flex h-[calc(100vh-12rem)] gap-4">
      {/* Sidebar */}
      <Show when={showSidebar()}>
        <div class="w-64 flex-shrink-0 flex flex-col gap-4">
          {/* New Chat Button */}
          <Button onClick={startNewSession} class="w-full">
            <svg
              class="w-4 h-4 mr-2"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M12 4v16m8-8H4"
              />
            </svg>
            New Chat
          </Button>

          {/* Context Settings */}
          <Card>
            <CardHeader class="py-3">
              <CardTitle class="text-sm">Preferences</CardTitle>
            </CardHeader>
            <CardContent class="space-y-3">
              <div>
                <label class="text-xs text-muted-foreground">Providers</label>
                <div class="flex gap-1 mt-1">
                  <For each={["aws", "azure", "gcp"] as CloudProvider[]}>
                    {(provider) => (
                      <button
                        onClick={() => toggleProvider(provider)}
                        class={`px-2 py-1 text-xs rounded ${
                          context().preferred_providers.includes(provider)
                            ? "bg-primary text-primary-foreground"
                            : "bg-muted text-muted-foreground"
                        }`}
                      >
                        {provider.toUpperCase()}
                      </button>
                    )}
                  </For>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Session History */}
          <Card class="flex-1 overflow-hidden">
            <CardHeader class="py-3">
              <CardTitle class="text-sm">History</CardTitle>
            </CardHeader>
            <CardContent class="p-0 overflow-y-auto max-h-64">
              <For each={sessions()}>
                {(s) => (
                  <button
                    onClick={() => selectSession(s)}
                    class={`w-full text-left px-4 py-2 text-sm hover:bg-muted truncate ${
                      session()?.id === s.id ? "bg-muted" : ""
                    }`}
                  >
                    {s.title || "Untitled Chat"}
                  </button>
                )}
              </For>
              <Show when={sessions().length === 0}>
                <p class="px-4 py-2 text-sm text-muted-foreground">
                  No previous chats
                </p>
              </Show>
            </CardContent>
          </Card>
        </div>
      </Show>

      {/* Main Chat Area */}
      <div class="flex-1 flex flex-col min-w-0">
        {/* Toggle Sidebar Button */}
        <div class="mb-2">
          <button
            onClick={() => setShowSidebar(!showSidebar())}
            class="p-2 text-muted-foreground hover:text-foreground"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
            </svg>
          </button>
        </div>

        {/* Messages */}
        <Card class="flex-1 overflow-hidden flex flex-col">
          <CardContent class="flex-1 overflow-y-auto p-4 space-y-4">
            <Show
              when={messages().length > 0}
              fallback={
                <div class="flex flex-col items-center justify-center h-full text-center">
                  <div class="mb-8">
                    <svg
                      class="w-16 h-16 text-muted-foreground mx-auto mb-4"
                      fill="none"
                      stroke="currentColor"
                      viewBox="0 0 24 24"
                    >
                      <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="1.5"
                        d="M8 10h.01M12 10h.01M16 10h.01M9 16H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-5l-5 5v-5z"
                      />
                    </svg>
                    <h2 class="text-xl font-semibold mb-2">
                      FinOps AI Assistant
                    </h2>
                    <p class="text-muted-foreground max-w-md">
                      I can help you optimize cloud costs, compare pricing across
                      providers, and recommend infrastructure configurations.
                    </p>
                  </div>
                  <div class="grid grid-cols-2 gap-2 max-w-lg">
                    <For each={EXAMPLE_PROMPTS}>
                      {(prompt) => (
                        <button
                          onClick={() => handleExampleClick(prompt)}
                          class="p-3 text-sm text-left rounded-lg border hover:bg-muted transition-colors"
                        >
                          {prompt}
                        </button>
                      )}
                    </For>
                  </div>
                </div>
              }
            >
              <For each={messages()}>
                {(message) => (
                  <div
                    class={`flex ${
                      message.role === "user" ? "justify-end" : "justify-start"
                    }`}
                  >
                    <div
                      class={`max-w-[80%] rounded-lg px-4 py-2 ${
                        message.role === "user"
                          ? "bg-primary text-primary-foreground"
                          : "bg-muted"
                      }`}
                    >
                      {/* Tool Calls */}
                      <Show when={message.toolCalls && message.toolCalls.length > 0}>
                        <div class="mb-2 space-y-2">
                          <For each={message.toolCalls}>
                            {(tool) => (
                              <div class="text-xs bg-background/50 rounded p-2">
                                <div class="flex items-center gap-2 font-medium">
                                  <svg
                                    class="w-3 h-3"
                                    fill="none"
                                    stroke="currentColor"
                                    viewBox="0 0 24 24"
                                  >
                                    <path
                                      stroke-linecap="round"
                                      stroke-linejoin="round"
                                      stroke-width="2"
                                      d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
                                    />
                                    <path
                                      stroke-linecap="round"
                                      stroke-linejoin="round"
                                      stroke-width="2"
                                      d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                                    />
                                  </svg>
                                  {tool.name}
                                </div>
                                <Show when={tool.result}>
                                  <div class="mt-1 text-muted-foreground truncate">
                                    {tool.result}
                                  </div>
                                </Show>
                              </div>
                            )}
                          </For>
                        </div>
                      </Show>

                      {/* Message Content */}
                      <div class="whitespace-pre-wrap">{message.content}</div>
                    </div>
                  </div>
                )}
              </For>

              {/* Loading Indicator */}
              <Show when={isLoading() && currentToolCall()}>
                <div class="flex justify-start">
                  <div class="bg-muted rounded-lg px-4 py-2 flex items-center gap-2">
                    <svg class="w-4 h-4 animate-spin" viewBox="0 0 24 24">
                      <circle
                        class="opacity-25"
                        cx="12"
                        cy="12"
                        r="10"
                        stroke="currentColor"
                        stroke-width="4"
                        fill="none"
                      />
                      <path
                        class="opacity-75"
                        fill="currentColor"
                        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                      />
                    </svg>
                    <span class="text-sm text-muted-foreground">
                      Running {currentToolCall()}...
                    </span>
                  </div>
                </div>
              </Show>

              <div ref={messagesEndRef} />
            </Show>
          </CardContent>

          {/* Input Area */}
          <div class="border-t p-4">
            <div class="flex gap-2">
              <Input
                value={input()}
                onInput={(e) => setInput(e.currentTarget.value)}
                onKeyDown={handleKeyDown}
                placeholder="Ask about cloud costs, pricing, or optimization..."
                disabled={isLoading()}
                class="flex-1"
              />
              <Button onClick={sendMessage} disabled={isLoading() || !input().trim()}>
                <Show
                  when={!isLoading()}
                  fallback={
                    <svg class="w-4 h-4 animate-spin" viewBox="0 0 24 24">
                      <circle
                        class="opacity-25"
                        cx="12"
                        cy="12"
                        r="10"
                        stroke="currentColor"
                        stroke-width="4"
                        fill="none"
                      />
                      <path
                        class="opacity-75"
                        fill="currentColor"
                        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                      />
                    </svg>
                  }
                >
                  <svg
                    class="w-4 h-4"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8"
                    />
                  </svg>
                </Show>
              </Button>
            </div>
          </div>
        </Card>
      </div>
    </div>
  );
}
