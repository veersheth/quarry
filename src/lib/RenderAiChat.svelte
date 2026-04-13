<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { aiSubmitQuery } from "../stores/search";

  const MODEL = {
    name: "llama-3.3-70b-versatile",
    dashboard: "https://console.groq.com/",
  };

  let apiKey = invoke<string>("get_groq_api_key");

  let response = "";
  let loading = false;
  let abort: AbortController | null = null;

  // usage tracking
  let lastTokens: number | null = null;
  let totalUsed = 0;

  $: if ($aiSubmitQuery) stream($aiSubmitQuery);

  async function stream(q: string) {
    abort?.abort();
    abort = new AbortController();

    response = "";
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
You are a concise assistant embedded in a small spotlight-style launcher. 
Be brief. Use markdown only when helpful. Avoid long prose (~200 words for most requests, unless they need more explanation).
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

            // usage tracking (only present at end in some responses)
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
      loading = false;
      aiSubmitQuery.set("");
    }
  }

  function md(text: string): string {
    let h = text
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;");

    h = h.replace(
      /```(\w*)\n?([\s\S]*?)```/g,
      (_, _l, c) => `<pre><code>${c.trimEnd()}</code></pre>`,
    );

    h = h.replace(/`([^`]+)`/g, "<code>$1</code>");
    h = h.replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>");
    h = h.replace(/\*([^*\n]+)\*/g, "<em>$1</em>");

    h = h.replace(/^### (.+)$/gm, "<h3>$1</h3>");
    h = h.replace(/^## (.+)$/gm, "<h2>$1</h2>");
    h = h.replace(/^# (.+)$/gm, "<h1>$1</h1>");

    h = h.replace(
      /((?:^[-*] .+\n?)+)/gm,
      (b) =>
        `<ul>${b
          .trim()
          .split("\n")
          .map((l) => `<li>${l.replace(/^[-*] /, "")}</li>`)
          .join("")}</ul>`,
    );

    h = h.replace(
      /((?:^\d+\. .+\n?)+)/gm,
      (b) =>
        `<ol>${b
          .trim()
          .split("\n")
          .map((l) => `<li>${l.replace(/^\d+\. /, "")}</li>`)
          .join("")}</ol>`,
    );

    return h
      .split(/\n\s*\n/)
      .map((p) => {
        p = p.trim();
        if (!p) return "";
        if (/^<(h\d|ul|ol|pre)/.test(p)) return p;
        return `<p>${p.replace(/\n/g, "<br>")}</p>`;
      })
      .filter(Boolean)
      .join("\n");
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
      to {
        --ai-deg: 360deg;
      }
    }

    .ai-box-glowing {
      animation: ai-spin 3s linear infinite;
    }

    .ai-box-filling {
      animation: ai-spin 1.8s linear infinite;
    }
  </style>
</svelte:head>

{#await apiKey then key}
  <div class="outer">
    <div class="inner">
      {#if !key}
        <p class="dim">
          Add <code>groq_api_key = "gsk_..."</code> to ~/.config/quarry/config.toml
        </p>
      {:else if loading && !response}
        <div class="dots"><span /><span /><span /></div>
      {:else if response}
        <div class="md">{@html md(response)}</div>
      {:else}
        <p class="dim">Press Enter to ask</p>
      {/if}
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
  }

  .inner {
    margin: 20px;
    border: 1px solid rgba(255, 255, 255, 0.3);
    border-radius: 20px;
    padding: 16px 18px;
    max-height: 400px;
    overflow-y: auto;
  }

  .dim {
    opacity: 0.4;
    margin: 0;
  }

  .md :global(p) {
    margin: 0 0 1rem;
    line-height: 1.6;
  }

  .md :global(code) {
    font-family: "JetBrains Mono", monospace;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 4px;
    padding: 0.1rem 0.3rem;
  }

  .md :global(pre) {
    background: rgba(0, 0, 0, 0.3);
    border-radius: 8px;
    padding: 12px;
    overflow-x: auto;
  }

  .dots {
    display: flex;
    gap: 5px;
  }

  .dots span {
    width: 6px;
    height: 6px;
    background: #60a5fa;
    border-radius: 50%;
    animation: pulse 1.4s infinite ease-in-out;
  }

  .footer {
    padding: 16px;
    display: flex;
    gap: 10px;
    align-items: center;
    justify-content: space-between;
    font-size: 0.8rem;
    opacity: 0.6;
    border-top: 1px solid rgba(255, 255, 255, 0.4);
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
