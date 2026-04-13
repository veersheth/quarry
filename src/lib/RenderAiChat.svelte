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
    let h = text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
    h = h.replace(/```(\w*)\n?([\s\S]*?)```/g, (_, _l, c) => `<pre><code>${c.trimEnd()}</code></pre>`);
    h = h.replace(/`([^`]+)`/g, "<code>$1</code>");
    h = h.replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>");
    h = h.replace(/\*([^*\n]+)\*/g, "<em>$1</em>");
    h = h.replace(/^### (.+)$/gm, "<h3>$1</h3>");
    h = h.replace(/^## (.+)$/gm,  "<h2>$1</h2>");
    h = h.replace(/^# (.+)$/gm,   "<h1>$1</h1>");
    h = h.replace(/((?:^[-*] .+\n?)+)/gm, b => `<ul>${b.trim().split("\n").map(l=>`<li>${l.replace(/^[-*] /,"")}</li>`).join("")}</ul>`);
    h = h.replace(/((?:^\d+\. .+\n?)+)/gm, b => `<ol>${b.trim().split("\n").map(l=>`<li>${l.replace(/^\d+\. /,"")}</li>`).join("")}</ol>`);
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
  <div class="outer">
    <div class="box" class:glowing={loading}>
      <div class="inner">
        {#if !key}
          <p class="dim">Add <code>groq_api_key = "gsk_..."</code> to ~/.config/quarry/config.toml</p>
        {:else if loading && !response}
          <div class="dots">
            <span /><span /><span />
          </div>
        {:else if response}
          <div class="md">{@html md(response)}</div>
        {:else}
          <p class="dim">Press Enter to ask</p>
        {/if}
      </div>
    </div>
  </div>
{/await}

<style>
  .outer {
    padding: 14px;
  }

  .box {
    position: relative;
    border-radius: 12px;
    padding: 2px; 
    background: rgba(255, 255, 255, 0.04);
    overflow: hidden; /* Important: crops the large spinning pseudo-element */
    isolation: isolate; /* Creates a new stacking context */
  }

  /* spinning gradient */
  .box::before {
    content: "";
    position: absolute;
    inset: -150%; 
    background: conic-gradient(
      from 0deg,
      transparent 0deg,
      #3b82f6 60deg,
      #60a5fa 120deg,
      #818cf8 180deg,
      transparent 240deg
    );
    opacity: 0;
    transition: opacity 0.4s ease;
    z-index: -2;
  }

  .box::after {
    content: "";
    position: absolute;
    inset: 2px; /* Border width */
    background: #141418;
    border-radius: 10px;
    z-index: -1;
  }

  .box.glowing::before {
    opacity: 1;
    animation: spin 3s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .inner {
    position: relative;
    z-index: 1;
    padding: 16px 18px;
    max-height: 400px; 
    overflow-y: auto;
    scrollbar-width: thin;
    scrollbar-color: rgba(255,255,255,0.1) transparent;
  }

  .dim {
    opacity: 0.4;
    margin: 0;
  }

  .md :global(p)            { margin: 0 0 1rem; line-height: 1.6; }
  .md :global(p:last-child) { margin-bottom: 0; }
  .md :global(h1), .md :global(h2), .md :global(h3) { font-weight: 600; margin: 1.2rem 0 0.4rem; }
  .md :global(h1:first-child), .md :global(h2:first-child), .md :global(h3:first-child) { margin-top: 0; }
  .md :global(ul), .md :global(ol) { margin: 0 0 1rem; padding-left: 1.4rem; }
  .md :global(li) { margin-bottom: 0.3rem; line-height: 1.5; }
  .md :global(strong) { font-weight: 600; }
  .md :global(em) { font-style: italic; }
  .md :global(code) {
    font-family: "JetBrains Mono", ui-monospace, monospace;
    background: rgba(255,255,255,0.1);
    border-radius: 4px;
    padding: 0.1rem 0.3rem;
    font-size: 0.9em;
  }
  .md :global(pre) {
    background: rgba(0,0,0,0.3);
    border: 1px solid rgba(255,255,255,0.08);
    border-radius: 8px;
    padding: 12px 14px;
    margin: 0.8rem 0;
    overflow-x: auto;
  }
  .md :global(pre code) { background: transparent; padding: 0; display: block; line-height: 1.5; }

  .dots { display: flex; gap: 5px; align-items: center; }
  .dots span {
    width: 6px; height: 6px;
    background: #3b82f6;
    border-radius: 50%;
    opacity: 0.3;
    animation: pulse 1.4s infinite ease-in-out;
  }
  .dots span:nth-child(2) { animation-delay: 0.2s; }
  .dots span:nth-child(3) { animation-delay: 0.4s; }

  @keyframes pulse {
    0%, 100% { opacity: 0.3; transform: scale(1); }
    50%       { opacity: 1; transform: scale(1.2); }
  }
</style>
