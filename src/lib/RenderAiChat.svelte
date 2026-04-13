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
          messages: [{ role: "user", content: q }],
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
</script>

{#await apiKey then key}
  <div class="ai">
    {#if !key}
      <p>Add <code>groq_api_key = "gsk_..."</code> to ~/.config/quarry/config.toml</p>
    {:else if loading && !response}
      <p>...</p>
    {:else if response}
      <p class="response">{response}</p>
    {:else}
      <p>Press Enter to ask</p>
    {/if}
  </div>
{/await}

<style>
  .ai {
    padding: 14px 20px;
  }

  p {
    margin: 0;
    line-height: 1.6;
    white-space: pre-wrap;
  }

  code {
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 4px;
    padding: 1px 5px;
  }
</style>
