<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { unified } from "unified";
  import remarkParse from "remark-parse";
  import remarkGfm from "remark-gfm";
  import remarkRehype from "remark-rehype";
  import rehypeSanitize from "rehype-sanitize";
  import rehypeStringify from "rehype-stringify";
  import { aiSubmitQuery } from "../stores/search";

  const processor = unified()
    .use(remarkParse)
    .use(remarkGfm)
    .use(remarkRehype)
    .use(rehypeSanitize)
    .use(rehypeStringify);

  async function parseMarkdown(text: string): Promise<string> {
    return String(await processor.process(text));
  }

  const MODEL = {
    name: "llama-3.3-70b-versatile",
    dashboard: "https://console.groq.com/",
  };

  const apiKey = invoke<string>("get_groq_api_key");

  let response = "";
  let rendered = "";
  let loading = false;
  let hasError = false;
  let errorLabel = "";
  let abort: AbortController | null = null;
  let timeoutId: ReturnType<typeof setTimeout> | null = null;
  let isTimeout = false;
  let lastTokens: number | null = null;
  let glowState: "success" | "error" | "" = "";

  function triggerGlow(type: "success" | "error") {
    glowState = type;
    setTimeout(() => (glowState = ""), 2000);
  }

  function setError(label: string, detail: string) {
    errorLabel = label;
    response = detail;
    hasError = true;
    triggerGlow("error");
  }

  $: if ($aiSubmitQuery) stream($aiSubmitQuery);

  async function stream(q: string) {
    abort?.abort();
    abort = new AbortController();

    response = "";
    rendered = "";
    errorLabel = "";
    loading = true;
    hasError = false;
    lastTokens = null;
    isTimeout = false;

    if (timeoutId) clearTimeout(timeoutId);
    timeoutId = setTimeout(() => {
      isTimeout = true;
      abort?.abort();
    }, 8000);

    try {
      const key = await apiKey;

      let res: Response;
      try {
        res = await fetch("https://api.groq.com/openai/v1/chat/completions", {
          method: "POST",
          signal: abort.signal,
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${key}`,
          },
          body: JSON.stringify({
            model: MODEL.name,
            stream: true,
            messages: [
              {
                role: "system",
                content: `You are a concise assistant called Quarry. Give correct answers only, be brief and to the point. Only be playful when the prompt calls for it. Avoid long prose (~200 words for most requests, unless more explanation is needed).`,
              },
              { role: "user", content: q },
            ],
          }),
        });
      } catch (e: any) {
        if (e.name === "AbortError" && isTimeout) {
          setError("Request timed out", "The API is either too slow or unreachable right now.");
          return;
        }
        if (e.name === "AbortError") return;
        if (!navigator.onLine) {
          setError("No network", "Check your internet connection.");
        } else {
          setError("Connection failed", "Groq API isn't reachable.");
        }
        return;
      }

      if (!res.ok) {
        let detail = "";
        try {
          const body = await res.json();
          detail = body?.error?.message ?? "";
        } catch {}

        const messages: Record<number, [string, string]> = {
          401: ["Invalid API key", detail || "Your Groq API key was rejected. Check ~/.config/quarry/config.toml."],
          403: ["Access denied", detail || "Your key doesn't have permission to use this model."],
          429: ["Rate limited", detail || "Too many requests or quota exceeded. Try again shortly."],
          500: ["Groq server error", detail || "API down."],
          502: ["Groq server error", detail || "API down."],
          503: ["Groq server error", detail || "API down."],
        };

        const [label, msg] = messages[res.status] ?? [`HTTP ${res.status}`, detail || "Unexpected error."];
        setError(label, msg);
        return;
      }

      const reader = res.body!.getReader();
      const dec = new TextDecoder();
      let buf = "";

      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        buf += dec.decode(value, { stream: true });
        const lines = buf.split("\n");
        buf = lines.pop() ?? "";

        for (const line of lines) {
          if (!line.startsWith("data: ")) continue;
          const d = line.slice(6).trim();
          if (d === "[DONE]") break;

          try {
            const parsed = JSON.parse(d);
            response += parsed.choices?.[0]?.delta?.content ?? "";
            rendered = await parseMarkdown(response);
            if (parsed.usage?.total_tokens) lastTokens = parsed.usage.total_tokens;
          } catch {}
        }
      }
    } catch (e: any) {
      if (e.name === "AbortError" && isTimeout) {
        setError("Request timed out", "Request timed out after 8 seconds.");
        return;
      }
      if (e.name === "AbortError") return;
      setError("Unexpected error", e.message ?? "Something went wrong.");
    } finally {
      if (timeoutId) { clearTimeout(timeoutId); timeoutId = null; }
      if (!hasError) {
        rendered = await parseMarkdown(response);
        triggerGlow("success");
      }
      loading = false;
      aiSubmitQuery.set("");
    }
  }
</script>

<svelte:head>
  <style>
    @property --ai-deg {
      syntax: "<angle>";
      inherits: true;
      initial-value: 0deg;
    }

    @keyframes ai-spin {
      to { --ai-deg: 360deg; }
    }

    @keyframes pulse {
      0%, 100% { opacity: 0.4; transform: scale(1); }
      50%       { opacity: 1;   transform: scale(1.2); }
    }

    @keyframes answer-glow {
      0%   { box-shadow: 0 0 0px  rgba(59, 130, 246, 0); }
      30%  { box-shadow: 0 0 28px rgba(64, 0, 255, 0.8); }
      100% { box-shadow: 0 0 0px  rgba(59, 130, 246, 0); }
    }

    @keyframes error-glow {
      0%   { box-shadow: 0 0 0px  rgba(226, 75, 74, 0); }
      30%  { box-shadow: 0 0 28px rgba(226, 75, 74, 0.8); }
      100% { box-shadow: 0 0 0px  rgba(226, 75, 74, 0); }
    }

    @keyframes shake {
      0%, 100% { transform: translateX(0);  }
      20%       { transform: translateX(-6px); }
      40%       { transform: translateX(6px);  }
      60%       { transform: translateX(-4px); }
      80%       { transform: translateX(4px);  }
    }
  </style>
</svelte:head>

{#await apiKey then key}
  <div class="outer">
    <div
      class="ai-container"
      class:is-loading={loading}
      class:has-response={rendered && !loading}
      class:just-answered={glowState === "success"}
      class:just-errored={glowState === "error"}
    >
      <div class="inner">
        {#if !key}
          <p class="dim">
            Add <code>groq_api_key = "gsk_..."</code> to ~/.config/quarry/config.toml
          </p>
        {:else if loading && !rendered}
          <div class="dots"><span /><span /><span /></div>
        {:else if hasError}
          <div class="error-block">
            <span class="error-badge">{errorLabel}</span>
            <p class="error-msg">{response}</p>
          </div>
        {:else if rendered}
          <div class="md">{@html rendered}</div>
        {:else}
          <p class="dim">Press Enter to ask</p>
        {/if}
      </div>
    </div>

    {#if key}
      <div class="footer">
        <span class="model">{MODEL.name}</span>
        <div class="footer-right">
          {#if lastTokens !== null}
            <span class="usage">{lastTokens} tokens</span>
          {/if}
          <a href={MODEL.dashboard} target="_blank" rel="noopener noreferrer" class="link">
            Dashboard ↗
          </a>
        </div>
      </div>
    {/if}
  </div>
{/await}

<style>
  .outer {
    height: 100%;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    font-size: 1rem;
  }

  .ai-container {
    position: relative;
    margin: 20px;
    padding: 1px;
    border-radius: 12px;
    background: rgba(255, 255, 255, 0.05);
    transition: background 0.6s cubic-bezier(0.22, 1, 0.36, 1),
                box-shadow  0.6s cubic-bezier(0.22, 1, 0.36, 1);
  }

  .ai-container::before {
    content: "";
    position: absolute;
    inset: -1px;
    border-radius: 20px;
    padding: 2px;
    background: conic-gradient(
      from var(--ai-deg),
      transparent 0deg,
      #3b82f6 120deg,
      #60a5fa 180deg,
      #93c5fd 240deg,
      transparent 360deg
    );
    -webkit-mask: linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0);
            mask: linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0);
    -webkit-mask-composite: xor;
            mask-composite: exclude;
    opacity: 0;
    transition: opacity 0.4s ease;
  }

  .ai-container.is-loading::before {
    opacity: 1;
    animation: ai-spin 1.2s linear infinite;
  }

  .ai-container.has-response {
    background: rgba(59, 130, 246, 0.08);
    box-shadow: 0 0 30px rgba(59, 130, 246, 0.15);
  }

  .ai-container.has-response::before {
    opacity: 0.5;
    animation: ai-spin 8s linear infinite;
  }

  .ai-container.just-answered {
    animation: answer-glow 2s ease-out forwards;
  }

  .ai-container.just-errored::before {
    background: conic-gradient(
      from var(--ai-deg),
      transparent 0deg,
      #e24b4a 120deg,
      #f09595 180deg,
      #fcc     240deg,
      transparent 360deg
    );
    opacity: 1;
    animation: ai-spin 1.8s linear infinite;
  }

  .ai-container.just-errored {
    animation: error-glow 2s ease-out forwards;
  }

  .ai-container.just-errored .inner {
    animation: shake 0.45s ease-out;
  }

  .inner {
    border-radius: 12px;
    padding: 16px 18px;
    max-height: 400px;
    overflow-y: auto;
    position: relative;
    z-index: 1;
  }

  .dim {
    opacity: 0.4;
    margin: 0;
  }

  .error-block {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .error-badge {
    display: inline-flex;
    align-items: center;
    font-size: 0.7rem;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    background: rgba(226, 75, 74, 0.12);
    color: #f09595;
    border: 1px solid rgba(226, 75, 74, 0.3);
    border-radius: 6px;
    padding: 3px 10px;
    width: fit-content;
  }

  .error-msg {
    font-size: 0.82rem;
    font-family: "JetBrains Mono", monospace;
    color: #f09595;
    margin: 0;
    opacity: 0.8;
    word-break: break-all;
  }

  .md :global(p)     { margin: 0 0 0.75rem; line-height: 1.6; }
  .md :global(h1),
  .md :global(h2),
  .md :global(h3),
  .md :global(h4),
  .md :global(h5),
  .md :global(h6)    { margin: 1rem 0 0.4rem; line-height: 1.3; font-weight: 600; }
  .md :global(ul),
  .md :global(ol)    { margin: 0 0 0.75rem 1.25rem; padding: 0; }
  .md :global(li)    { margin-bottom: 0.25rem; line-height: 1.5; }

  .md :global(code) {
    font-family: "JetBrains Mono", monospace;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 4px;
    padding: 0.1rem 0.3rem;
    font-size: 0.875em;
  }

  .md :global(pre) {
    background: rgba(0, 0, 0, 0.3);
    border-radius: 8px;
    padding: 12px;
    overflow-x: auto;
    margin: 0 0 0.75rem;
  }

  .md :global(pre code) { background: none; padding: 0; font-size: 0.875em; }

  .md :global(blockquote) {
    border-left: 3px solid #3b82f6;
    margin: 0 0 0.75rem;
    padding: 0.25rem 0 0.25rem 1rem;
    opacity: 0.8;
  }

  .md :global(table)  { width: 100%; border-collapse: collapse; margin: 0 0 0.75rem; font-size: 0.9em; }
  .md :global(th),
  .md :global(td)     { border: 1px solid rgba(255, 255, 255, 0.15); padding: 6px 10px; text-align: left; }
  .md :global(th)     { background: rgba(255, 255, 255, 0.07); font-weight: 600; }
  .md :global(hr)     { border: none; border-top: 1px solid rgba(255, 255, 255, 0.1); margin: 1rem 0; }
  .md :global(a)      { color: #60a5fa; text-decoration: none; }
  .md :global(a:hover){ text-decoration: underline; }
  .md :global(del)    { opacity: 0.6; }

  .dots          { display: flex; gap: 5px; padding: 10px 0; }
  .dots span     { width: 8px; height: 8px; background: #60a5fa; border-radius: 50%; animation: pulse 1.4s infinite ease-in-out; }
  .dots span:nth-child(2) { animation-delay: 0.2s; }
  .dots span:nth-child(3) { animation-delay: 0.4s; }

  .footer {
    padding: 16px;
    display: flex;
    gap: 10px;
    align-items: center;
    justify-content: space-between;
    font-size: 0.8rem;
    opacity: 0.6;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
  }

  .footer-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .model {
    font-weight: 500;
    font-family: "JetBrainsMono Nerd Font", "Cascadia Mono", monospace;
  }

  .link {
    text-decoration: none;
    color: #60a5fa;
    font-family: "JetBrainsMono Nerd Font", "Cascadia Mono", monospace;
  }

  .usage {
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 12px;
    padding: 6px 10px;
    font-family: "JetBrainsMono Nerd Font", "Cascadia Mono", monospace;
  }
</style>
