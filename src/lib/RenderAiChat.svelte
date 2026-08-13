<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import FooterBar from "./ui/FooterBar.svelte";
  import Chip from "./ui/Chip.svelte";
  import { unified } from "unified";
  import remarkParse from "remark-parse";
  import remarkGfm from "remark-gfm";
  import remarkRehype from "remark-rehype";
  import rehypeSanitize from "rehype-sanitize";
  import rehypeStringify from "rehype-stringify";
  import { aiSubmitQuery } from "../stores/search";
  import { addToast } from "../stores/toasts";
  import { onMount } from "svelte";

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
    name: "openai/gpt-oss-120b",
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

  async function copyResult() {
    if (!response) return;
    await navigator.clipboard.writeText(response);
    addToast("AI response copied to clipboard", "success");
  }

  // Animation state
  type GlowState = "idle" | "loading" | "streaming" | "success" | "error" | "settling";
  let glowState: GlowState = "idle";
  let beamVisible = false;
  let mounted = false;

  onMount(() => {
    setTimeout(() => {
      beamVisible = true;
      mounted = true;
    }, 120);
  });

  function setGlow(state: GlowState) {
    glowState = state;
  }

  function triggerSuccess() {
    setGlow("success");
    setTimeout(() => setGlow("idle"), 2400);
  }

  function triggerError() {
    setGlow("error");
    setTimeout(() => setGlow("settling"), 2400);
    setTimeout(() => setGlow("idle"), 3600);
  }

  function setError(label: string, detail: string) {
    errorLabel = label;
    response = detail;
    hasError = true;
    triggerError();
  }

  // Flicker the border on each streaming chunk
  let flickerTimeout: ReturnType<typeof setTimeout> | null = null;
  function onChunkArrived() {
    if (glowState === "streaming") {
      const el = document.querySelector(".ai-border") as HTMLElement | null;
      if (el) {
        el.style.animationDuration = "0.45s, 1.7s";
        if (flickerTimeout) clearTimeout(flickerTimeout);
        flickerTimeout = setTimeout(() => {
          el.style.animationDuration = "";
        }, 320);
      }
    }
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
    setGlow("loading");

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
          401: ["Invalid API key", detail || "Your Groq API key was rejected. Update it in Settings → General."],
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
      let firstChunk = true;

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
            const delta = parsed.choices?.[0]?.delta?.content ?? "";
            if (delta) {
              if (firstChunk) {
                firstChunk = false;
                setGlow("streaming");
              }
              response += delta;
              rendered = await parseMarkdown(response);
              onChunkArrived();
            }
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
        triggerSuccess();
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
    @property --ai-border-opacity {
      syntax: "<number>";
      inherits: true;
      initial-value: 0;
    }

    @keyframes ai-spin-idle    { to { --ai-deg: 360deg; } }
    @keyframes ai-spin-load    { to { --ai-deg: 360deg; } }
    @keyframes ai-spin-stream  { to { --ai-deg: 360deg; } }
    @keyframes ai-spin-success { to { --ai-deg: 360deg; } }
    @keyframes ai-spin-error   { to { --ai-deg: 360deg; } }

    @keyframes idle-breathe {
      0%, 100% { --ai-border-opacity: 0.13; }
      50%       { --ai-border-opacity: 0.30; }
    }
    @keyframes load-pulse {
      0%, 100% { --ai-border-opacity: 0.55; }
      50%       { --ai-border-opacity: 1;    }
    }
    @keyframes stream-flicker {
      0%, 100% { --ai-border-opacity: 0.60; }
      33%       { --ai-border-opacity: 1;    }
      66%       { --ai-border-opacity: 0.45; }
    }

    @keyframes success-bloom {
      0%   { box-shadow: 0 0 0px   rgba(99,102,241,0);    }
      18%  { box-shadow: 0 0 44px 10px rgba(99,102,241,0.52); }
      55%  { box-shadow: 0 0 24px 3px  rgba(99,102,241,0.26); }
      100% { box-shadow: 0 0 0px   rgba(99,102,241,0);    }
    }
    @keyframes error-bloom {
      0%   { box-shadow: 0 0 0px   rgba(220,38,38,0);    }
      18%  { box-shadow: 0 0 44px 8px  rgba(220,38,38,0.50); }
      55%  { box-shadow: 0 0 20px 2px  rgba(220,38,38,0.26); }
      100% { box-shadow: 0 0 8px  1px  rgba(220,38,38,0.12); }
    }
    @keyframes settling-dim {
      from { box-shadow: 0 0 8px 1px rgba(220,38,38,0.12); }
      to   { box-shadow: 0 0 0px rgba(220,38,38,0); }
    }

    @keyframes shake {
      0%,100% { transform: translateX(0); }
      15%     { transform: translateX(-7px) rotate(-0.4deg); }
      30%     { transform: translateX(7px)  rotate(0.4deg);  }
      50%     { transform: translateX(-5px); }
      65%     { transform: translateX(5px);  }
      80%     { transform: translateX(-2px); }
    }

    @keyframes dot-pulse {
      0%,80%,100% { transform: scale(0.65); opacity: 0.35; }
      40%          { transform: scale(1.2);  opacity: 1;    }
    }

    @keyframes beam-slide {
      0%   { transform: translateX(-100%); opacity: 0; }
      10%  { opacity: 1; }
      80%  { opacity: 1; }
      100% { transform: translateX(100%);  opacity: 0; }
    }
    @keyframes beam-glow-fade {
      0%   { opacity: 0; }
      15%  { opacity: 1; }
      75%  { opacity: 1; }
      100% { opacity: 0; }
    }
  </style>
</svelte:head>

{#await apiKey then key}
  <div class="outer">

    <!-- ── Top beam ───────────────────────────────── -->
    <div class="beam-rail" aria-hidden="true">
      <div class="beam-line" class:beam-active={beamVisible}></div>
      <div class="beam-glow" class:beam-active={beamVisible}></div>
    </div>

    <!-- ── Main card ──────────────────────────────── -->
    <div
      class="ai-wrap"
      class:state-idle={glowState === "idle"}
      class:state-loading={glowState === "loading"}
      class:state-streaming={glowState === "streaming"}
      class:state-success={glowState === "success"}
      class:state-error={glowState === "error"}
      class:state-settling={glowState === "settling"}
    >
      <div class="ai-border" aria-hidden="true"></div>

      <div
        class="inner"
        class:shake={glowState === "error"}
      >
        {#if !key}
          <p class="dim">
            Add a Groq API key in Settings → General to enable AI chat.
          </p>
        {:else if loading && !rendered}
          <div class="dots">
            <span></span><span></span><span></span>
          </div>
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
      <FooterBar>
        <Chip mono>{MODEL.name}</Chip>
        <div class="footer-right">
          {#if lastTokens !== null}
            <Chip>{lastTokens} tokens</Chip>
          {/if}
          {#if response && !loading}
            <Chip as="button" on:click={copyResult}>Copy</Chip>
          {/if}
          <Chip href={MODEL.dashboard}>Dashboard ↗</Chip>
        </div>
      </FooterBar>
    {/if}
  </div>
{/await}

<style>
  .outer {
    height: 100%;
    display: flex;
    flex-direction: column;
    font-size: 1em;
  }

  .beam-rail {
    position: relative;
    width: 100%;
    height: 2px;
    overflow: hidden;
    margin-bottom: -10px;
    z-index: 10;
  }

  .beam-line {
    position: absolute;
    top: 0;
    left: 0;
    width: 60%;
    height: 2px;
    background: linear-gradient(
      90deg,
      transparent 0%,
      rgba(148,130,255,0.9) 20%,
      #c4b5fd 45%,
      #fff     50%,
      #c4b5fd 55%,
      rgba(148,130,255,0.4) 80%,
      transparent 100%
    );
    border-radius: 20px;
    transform: translateX(-100%);
    opacity: 0;
  }
  .beam-line.beam-active {
    animation: beam-slide 1.6s cubic-bezier(0.4, 0, 0.2, 1) 0.1s forwards;
  }

  .beam-glow {
    position: absolute;
    top: -6px;
    left: 0;
    width: 100%;
    height: 14px;
    background: radial-gradient(
      ellipse 55% 100% at 50% 0%,
      rgba(139,92,246,0.35) 0%,
      transparent 100%
    );
    opacity: 0;
    pointer-events: none;
  }
  .beam-glow.beam-active {
    animation: beam-glow-fade 1.8s ease-out 0.1s forwards;
  }

  .ai-wrap {
    position: relative;
    flex: 1;
    min-height: 0;
    margin: 20px 20px 0;
    padding: 1.5px;
    border-radius: 20px;
    background: rgba(128,128,128,0.06);
    transition:
      background       0.6s cubic-bezier(0.22,1,0.36,1),
      box-shadow       0.6s cubic-bezier(0.22,1,0.36,1);
    display: flex;
    flex-direction: column;
  }

  .ai-border {
    position: absolute;
    inset: -1.5px;
    border-radius: 15.5px;
    padding: 2px;
    background: conic-gradient(
      from var(--ai-deg),
      transparent 0deg,
      #6366f1  90deg,
      #818cf8 170deg,
      #a5b4fc 240deg,
      transparent 360deg
    );
    mask: linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0);
    -webkit-mask: linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0);
    mask-composite: exclude;
    -webkit-mask-composite: xor;
    --ai-border-opacity: 0.18;
    opacity: var(--ai-border-opacity);
    animation:
      idle-breathe 3.8s ease-in-out infinite,
      ai-spin-idle 14s linear infinite;
    transition: opacity 0.5s ease;
  }

  .state-loading .ai-border {
    --ai-border-opacity: 0.9;
    animation:
      load-pulse 1.1s ease-in-out infinite,
      ai-spin-load 1.5s linear infinite;
  }
  .state-loading {
    background: rgba(99,102,241,0.05);
  }

  .state-streaming .ai-border {
    --ai-border-opacity: 0.72;
    animation:
      stream-flicker 0.7s ease-in-out infinite,
      ai-spin-stream 2.0s linear infinite;
  }
  .state-streaming {
    background: rgba(99,102,241,0.06);
  }

  .state-success .ai-border {
    --ai-border-opacity: 0.42;
    animation:
      ai-spin-success 9s linear infinite;
  }
  .state-success {
    background: rgba(99,102,241,0.07);
    animation: success-bloom 2.2s cubic-bezier(0.22,1,0.36,1) forwards;
  }

  .state-error .ai-border {
    --ai-border-opacity: 1;
    background: conic-gradient(
      from var(--ai-deg),
      transparent 0deg,
      #dc2626  80deg,
      #ef4444 160deg,
      #fca5a5 220deg,
      transparent 360deg
    );
    animation:
      load-pulse 0.9s ease-in-out infinite,
      ai-spin-error 1.6s linear infinite;
  }
  .state-error {
    background: rgba(220,38,38,0.05);
    animation: error-bloom 2.4s cubic-bezier(0.22,1,0.36,1) forwards;
  }

  .state-settling .ai-border {
    --ai-border-opacity: 0.18;
    background: conic-gradient(
      from var(--ai-deg),
      transparent 0deg,
      #6366f1  90deg,
      #818cf8 170deg,
      #a5b4fc 240deg,
      transparent 360deg
    );
    animation:
      idle-breathe 3.8s ease-in-out infinite,
      ai-spin-idle 14s linear infinite;
  }
  .state-settling {
    background: rgba(220,38,38,0.025);
    animation: settling-dim 1.2s ease-out forwards;
  }

  .inner {
    border-radius: 12px;
    padding: 16px 18px;
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    position: relative;
    z-index: 1;
    user-select: text !important;
    -webkit-user-select: text !important;
    cursor: text;
  }

  .inner :global(::selection) {
    background: rgba(99,102,241,0.3);
    color: inherit;
  }

  .inner :global(::-moz-selection) {
    background: rgba(99,102,241,0.3);
    color: inherit;
  }

  .inner.shake {
    animation: shake 0.5s cubic-bezier(0.36, 0.07, 0.19, 0.97);
  }

  .dim {
    opacity: 0.38;
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
    font-size: 0.68em;
    font-weight: 600;
    letter-spacing: 0.09em;
    text-transform: uppercase;
    background: rgba(220,38,38,0.10);
    color: #f87171;
    border: 1px solid rgba(220,38,38,0.24);
    border-radius: 6px;
    padding: 3px 10px;
    width: fit-content;
  }

  .error-msg {
    font-size: 0.8em;
    font-family: 'JetBrainsMono Nerd Font', 'Fira Code', 'Cascadia Mono', monospace;
    color: #fca5a5;
    margin: 0;
    opacity: 0.85;
    word-break: break-all;
  }

  .md {
    cursor: text;
    user-select: text !important;
    -webkit-user-select: text !important;
  }
  .md :global(p)               { margin: 0 0 0.75rem; line-height: 1.65; cursor: text; }
  .md :global(h1),
  .md :global(h2),
  .md :global(h3),
  .md :global(h4),
  .md :global(h5),
  .md :global(h6)              { margin: 1rem 0 0.4rem; line-height: 1.3; font-weight: 600; cursor: text; }
  .md :global(ul),
  .md :global(ol)              { margin: 0 0 0.75rem 1.25rem; padding: 0; cursor: text; }
  .md :global(li)              { margin-bottom: 0.25rem; line-height: 1.5; cursor: text; }
  .md :global(code) {
    font-family: 'JetBrainsMono Nerd Font', 'Fira Code', 'Cascadia Mono', monospace;
    background: rgba(255,255,255,0.1);
    border-radius: 4px;
    padding: 0.1rem 0.3rem;
    font-size: 0.875em;
    cursor: text;
    user-select: text !important;
    -webkit-user-select: text !important;
  }
  .md :global(pre) {
    background: rgba(0,0,0,0.3);
    border-radius: 8px;
    padding: 12px;
    overflow-x: auto;
    margin: 0 0 0.75rem;
    cursor: text;
    user-select: text !important;
    -webkit-user-select: text !important;
    position: relative;
  }
  .md :global(pre):hover {
    background: rgba(0,0,0,0.4);
    transition: background 0.15s ease;
  }
  .md :global(pre code)        { background: none; padding: 0; font-size: 0.875em; cursor: text; }
  .md :global(blockquote) {
    border-left: 3px solid #6366f1;
    margin: 0 0 0.75rem;
    padding: 0.25rem 0 0.25rem 1rem;
    opacity: 0.8;
  }
  .md :global(table)           { width: 100%; border-collapse: collapse; margin: 0 0 0.75rem; font-size: 0.9em; }
  .md :global(th),
  .md :global(td)              { border: 1px solid rgba(255,255,255,0.14); padding: 6px 10px; text-align: left; }
  .md :global(th)              { background: rgba(255,255,255,0.07); font-weight: 600; }
  .md :global(hr)              { border: none; border-top: 1px solid rgba(255,255,255,0.1); margin: 1rem 0; }
  .md :global(a)               { color: #818cf8; text-decoration: none; }
  .md :global(a:hover)         { text-decoration: underline; }
  .md :global(del)             { opacity: 0.6; }

  .dots          { display: flex; gap: 6px; padding: 10px 0; align-items: center; }
  .dots span     { width: 8px; height: 8px; background: #818cf8; border-radius: 50%; animation: dot-pulse 1.4s ease-in-out infinite; }
  .dots span:nth-child(2) { animation-delay: 0.2s; }
  .dots span:nth-child(3) { animation-delay: 0.4s; }

  .footer-right {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 6px;
  }
</style>
