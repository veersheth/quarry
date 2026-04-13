<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { aiSubmitQuery } from "../stores/search";

  let apiKey = invoke<string>("get_groq_api_key");
  let response = "";
  let loading = false;
  let abort: AbortController | null = null;

  $: if ($aiSubmitQuery) stream($aiSubmitQuery);

  async function stream(q: string) {
    abort?.abort();
    abort = new AbortController();
    response = "";
    loading = true;
    try {
      const key = await apiKey;
      const res = await fetch("https://api.groq.com/openai/v1/chat/completions", {
        method: "POST",
        signal: abort.signal,
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${key}`,
        },
        body: JSON.stringify({
          model: "llama-3.3-70b-versatile",
          stream: true,
          messages: [
            {
              role: "system",
              content: "You are a concise assistant embedded in a small spotlight-style launcher. Be brief. Use markdown only when it genuinely helps (code blocks for code, bold for key terms). Avoid long prose. Most answers should be around 200 words or less.",
            },
            { role: "user", content: q },
          ],
        }),
      });

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
            response += JSON.parse(d).choices?.[0]?.delta?.content ?? ""; 
          } catch {}
        }
      }
    } catch (e: any) {
      if (e.name !== "AbortError") response = e.message;
    } finally {
      loading = false;
      aiSubmitQuery.set("");
    } 
  }

  function md(text: string): string {
    // Basic escaping
    let h = text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
    
    // Blocks: Code, Lists, Headers
    h = h.replace(/```(\w*)\n?([\s\S]*?)```/g, (_, _l, c) => `<pre><code>${c.trimEnd()}</code></pre>`);
    h = h.replace(/`([^`]+)`/g, "<code>$1</code>");
    h = h.replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>");
    h = h.replace(/\*([^*\n]+)\*/g, "<em>$1</em>");
    h = h.replace(/^### (.+)$/gm, "<h3>$1</h3>");
    h = h.replace(/^## (.+)$/gm,  "<h2>$1</h2>");
    h = h.replace(/^# (.+)$/gm,   "<h1>$1</h1>");
    
    // Lists
    h = h.replace(/((?:^[-*] .+\n?)+)/gm, b => `<ul>${b.trim().split("\n").map(l=>`<li>${l.replace(/^[-*] /,"")}</li>`).join("")}</ul>`);
    h = h.replace(/((?:^\d+\. .+\n?)+)/gm, b => `<ol>${b.trim().split("\n").map(l=>`<li>${l.replace(/^\d+\. /,"")}</li>`).join("")}</ol>`);

    // Paragraph handling: split by double newline, trim, wrap in <p> if not already a block
    return h.split(/\n\s*\n/)
      .map(p => {
        p = p.trim();
        if (!p) return "";
        if (/^<(h\d|ul|ol|pre)/.test(p)) return p;
        return `<p>${p.replace(/\n/g, "<br>")}</p>`;
      })
      .filter(Boolean)
      .join("\n");
  }
</script>

{#await apiKey then key}
  <div class="ai">
    {#if !key}
      <p class="dim">
        Add <code>groq_api_key = "gsk_..."</code> to ~/.config/quarry/config.toml
      </p>
    {:else if loading && !response}
      <div class="loading-state">
        <span class="dot"></span>
        <span class="dot"></span>
        <span class="dot"></span>
      </div>
    {:else if response}
      <div class="md">{@html md(response)}</div>
    {:else}
      <p class="dim">Press Enter to ask</p>
    {/if}
  </div>
{/await}

<style>
  .ai {
    padding: 16px 20px;
    max-height: 420px;
    overflow-y: auto;
    scrollbar-width: thin;
    font-size: 1.05rem;
    color: rgba(255, 255, 255, 0.9);
  }

  .dim {
    opacity: 0.4;
    font-size: 0.95rem;
  }

  /* Improved Paragraph Spacing */
  .md :global(p) {
    margin: 0 0 1.25rem; 
    line-height: 1.6;
  }

  .md :global(p:last-child) {
    margin-bottom: 0;
  }

  /* Headers */
  .md :global(h1), .md :global(h2), .md :global(h3) {
    color: #fff;
    font-weight: 600;
    margin: 1.5rem 0 0.5rem;
  }

  .md :global(h1:first-child), 
  .md :global(h2:first-child), 
  .md :global(h3:first-child) {
    margin-top: 0;
  }

  /* Lists */
  .md :global(ul), .md :global(ol) {
    margin: 0 0 1.25rem;
    padding-left: 1.4rem;
  }

  .md :global(li) {
    margin-bottom: 0.4rem;
  }

  /* Code & Blocks */
  .md :global(code) {
    font-family: "JetBrains Mono", ui-monospace, monospace;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 4px;
    padding: 0.1rem 0.3rem;
    font-size: 0.9em;
  }

  .md :global(pre) {
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    padding: 14px;
    margin: 1.25rem 0;
    overflow-x: auto;
  }

  .md :global(pre code) {
    background: transparent;
    padding: 0;
    display: block;
    line-height: 1.5;
  }

  /* Simple Loading Animation */
  .loading-state {
    display: flex;
    gap: 4px;
    padding: 4px 0;
  }
  .dot {
    width: 6px;
    height: 6px;
    background: currentColor;
    border-radius: 50%;
    opacity: 0.3;
    animation: pulse 1.4s infinite;
  }
  .dot:nth-child(2) { animation-delay: 0.2s; }
  .dot:nth-child(3) { animation-delay: 0.4s; }

  @keyframes pulse {
    0%, 100% { opacity: 0.3; }
    50% { opacity: 0.7; }
  }
</style>
