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
    const result = await processor.process(text);
    return String(result);
  }

  const MODEL = {
    name: "llama-3.3-70b-versatile",
    dashboard: "https://console.groq.com/",
  };

  let apiKey = invoke<string>("get_groq_api_key");

  let response = "";
  let rendered = "";
  let loading = false;
  let abort: AbortController | null = null;

  // usage tracking
  let lastTokens: number | null = null;
  let totalUsed = 0;

  // glow trigger
  let container: HTMLElement;
  function triggerGlow() {
    container?.classList.add("just-answered");
    setTimeout(() => container?.classList.remove("just-answered"), 2000);
  }

  $: if ($aiSubmitQuery) stream($aiSubmitQuery);

  async function stream(q: string) {
    abort?.abort();
    abort = new AbortController();

    response = "";
    rendered = "";
    loading = true;
    lastTokens = null;

    try {
      const key = await apiKey;

      const res = await fetch(
        "https://api.groq.com/openai/v1/chat/completions",
        {
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
                content: `
You are a concise assistant and you are called Quarry. You give correct answers only and are only playful when the prompt requires it (Such as a joke or a sarcastic request).
Otherwise be brief and to the point, no need for unnecessary banter. Avoid long prose (~200 words for most requests, unless they need more explanation).
                `,
              },
              { role: "user", content: q },
            ],
          }),
        },
      );

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

            if (parsed.usage?.total_tokens) {
              lastTokens = parsed.usage.total_tokens;
              totalUsed += parsed.usage.total_tokens;
            }
          } catch {}
        }
      }
    } catch (e: any) {
      if (e.name !== "AbortError") response = e.message;
    } finally {
      // Final parse to ensure fully clean output
      rendered = await parseMarkdown(response);
      loading = false;
      if (response) triggerGlow();
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
      50% { opacity: 1; transform: scale(1.2); }
    }

    @keyframes answer-glow {
      0%   { box-shadow: 0 0 0px  rgba(59, 130, 246, 0); }
      30%  { box-shadow: 0 0 28px rgba(64, 0, 255, 0.8); }
      100% { box-shadow: 0 0 0px  rgba(59, 130, 246, 0); }
    }
  </style>
</svelte:head>

{#await apiKey then key}
  <div class="outer">
    <div
      class="ai-container"
      bind:this={container}
      class:is-loading={loading}
      class:has-response={rendered && !loading}
    >
      <div class="inner">
        {#if !key}
          <p class="dim">
            Add <code>groq_api_key = "gsk_..."</code> to ~/.config/quarry/config.toml
          </p>
        {:else if loading && !rendered}
          <div class="dots"><span /><span /><span /></div>
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
        <div>
          {#if lastTokens !== null}
            <span class="usage">{lastTokens} tokens</span>
          {/if}
          
            <a
            href={MODEL.dashboard}
            target="_blank"
            rel="noopener noreferrer"
            class="link"
          >
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
    font-size: 1.0rem;
  }

  .ai-container {
    position: relative;
    margin: 20px;
    padding: 1px;
    border-radius: 12px;
    background: rgba(255, 255, 255, 0.05);
    transition: all 0.6s cubic-bezier(0.22, 1, 0.36, 1);
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

  .md :global(p) {
    margin: 0 0 0.75rem;
    line-height: 1.6;
  }

  .md :global(h1),
  .md :global(h2),
  .md :global(h3),
  .md :global(h4),
  .md :global(h5),
  .md :global(h6) {
    margin: 1rem 0 0.4rem;
    line-height: 1.3;
    font-weight: 600;
  }

  .md :global(ul),
  .md :global(ol) {
    margin: 0 0 0.75rem 1.25rem;
    padding: 0;
  }

  .md :global(li) {
    margin-bottom: 0.25rem;
    line-height: 1.5;
  }

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

  .md :global(pre code) {
    background: none;
    padding: 0;
    font-size: 0.875em;
  }

  .md :global(blockquote) {
    border-left: 3px solid #3b82f6;
    margin: 0 0 0.75rem;
    padding: 0.25rem 0 0.25rem 1rem;
    opacity: 0.8;
  }

  .md :global(table) {
    width: 100%;
    border-collapse: collapse;
    margin: 0 0 0.75rem;
    font-size: 0.9em;
  }

  .md :global(th),
  .md :global(td) {
    border: 1px solid rgba(255, 255, 255, 0.15);
    padding: 6px 10px;
    text-align: left;
  }

  .md :global(th) {
    background: rgba(255, 255, 255, 0.07);
    font-weight: 600;
  }

  .md :global(hr) {
    border: none;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
    margin: 1rem 0;
  }

  .md :global(a) {
    color: #60a5fa;
    text-decoration: none;
  }

  .md :global(a:hover) {
    text-decoration: underline;
  }

  .md :global(del) {
    opacity: 0.6;
  }

  .dots {
    display: flex;
    gap: 5px;
    padding: 10px 0;
  }

  .dots span {
    width: 8px;
    height: 8px;
    background: #60a5fa;
    border-radius: 50%;
    animation: pulse 1.4s infinite ease-in-out;
  }

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

  .model {
    font-weight: 500;
    font-family: "JetBrainsMono Nerd Font", "Cascadia Mono", monospace;
  }

  .link {
    margin: 0 10px;
    text-decoration: none;
    color: #60a5fa;
    font-family: "JetBrainsMono Nerd Font", "Cascadia Mono", monospace;
  }

  .usage {
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgb(255, 255, 255, 0.2);
    border-radius: 12px;
    padding: 6px 10px;
    font-family: "JetBrainsMono Nerd Font", "Cascadia Mono", monospace;
  }
</style>
