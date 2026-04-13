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
              content: "You are a concise assistant embedded in a small spotlight-style launcher. Be brief. Use markdown only when it genuinely helps (code blocks for code, bold for key terms). Avoid long prose. Most answers should be around 200 words or less, unless it is absolutely required for a longer explanation",
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
          try { response += JSON.parse(d).choices?.[0]?.delta?.content ?? ""; } catch {}
        }
      }
    } catch (e: any) {
      if (e.name !== "AbortError") response = e.message;
    } finally {
      loading = false;
    }
  }

  function md(text: string): string {
    let h = text.replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;");
    h = h.replace(/```(\w*)\n?([\s\S]*?)```/g, (_,_l,c) => `<pre><code>${c.trimEnd()}</code></pre>`);
    h = h.replace(/`([^`]+)`/g, "<code>$1</code>");
    h = h.replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>");
    h = h.replace(/\*([^*\n]+)\*/g, "<em>$1</em>");
    h = h.replace(/^### (.+)$/gm, "<h3>$1</h3>");
    h = h.replace(/^## (.+)$/gm,  "<h2>$1</h2>");
    h = h.replace(/^# (.+)$/gm,   "<h1>$1</h1>");
    h = h.replace(/((?:^[-*] .+\n?)+)/gm, b => `<ul>${b.trim().split("\n").map(l=>`<li>${l.replace(/^[-*] /,"")}</li>`).join("")}</ul>`);
    h = h.replace(/((?:^\d+\. .+\n?)+)/gm, b => `<ol>${b.trim().split("\n").map(l=>`<li>${l.replace(/^\d+\. /,"")}</li>`).join("")}</ol>`);
    h = h.split(/\n{2,}/).map(p => {
      p = p.trim();
      if (!p || /^<(h\d|ul|ol|pre)/.test(p)) return p;
      return `<p>${p.replace(/\n/g,"<br>")}</p>`;
    }).filter(Boolean).join("\n");
    return h;
  }
</script>

{#await apiKey then key}
  <div class="ai">
    {#if !key}
      <p class="dim">Add <code>groq_api_key = "gsk_..."</code> to ~/.config/quarry/config.toml</p>
    {:else if loading && !response}
      <p class="dim">…</p>
    {:else if response}
      <!-- svelte-ignore svelte/no-at-html-tags -->
      <div class="md">{@html md(response)}</div>
    {:else}
      <p class="dim">Press Enter to ask</p>
    {/if}
  </div>
{/await}

<style>
  .ai {
    padding: 12px 20px 16px;
    max-height: 400px;
    overflow-y: auto;
    scrollbar-width: thin;
  }

  p { margin: 0; line-height: 1.6; }
  .dim { opacity: 0.35; }

  .md :global(p)          { margin: 0 0 8px; line-height: 1.6; }
  .md :global(p:last-child) { margin-bottom: 0; }
  .md :global(h1)         { font-weight: 600; margin: 0 0 6px; }
  .md :global(h2)         { font-weight: 600; margin: 10px 0 4px; }
  .md :global(h3)         { font-weight: 600; margin: 8px 0 4px; }
  .md :global(ul), .md :global(ol) { margin: 4px 0 8px; padding-left: 18px; }
  .md :global(li)         { margin: 2px 0; line-height: 1.5; }
  .md :global(strong)     { font-weight: 600; }
  .md :global(em)         { font-style: italic; }
  .md :global(code) {
    font-family: ui-monospace, monospace;
    background: rgba(255,255,255,0.08);
    border: 1px solid rgba(255,255,255,0.12);
    border-radius: 4px;
    padding: 1px 5px;
  }
  .md :global(pre) {
    font-family: ui-monospace, monospace;
    background: rgba(0,0,0,0.3);
    border: 1px solid rgba(255,255,255,0.08);
    border-radius: 6px;
    padding: 10px 12px;
    margin: 6px 0;
    overflow-x: auto;
    line-height: 1.5;
    white-space: pre;
  }
  .md :global(pre code) {
    background: none;
    border: none;
    padding: 0;
  }
</style>
