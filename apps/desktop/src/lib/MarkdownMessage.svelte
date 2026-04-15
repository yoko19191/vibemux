<script lang="ts">
  import { marked } from "marked";
  import DOMPurify from "dompurify";
  import hljs from "highlight.js/lib/core";
  import bash from "highlight.js/lib/languages/bash";
  import css from "highlight.js/lib/languages/css";
  import javascript from "highlight.js/lib/languages/javascript";
  import json from "highlight.js/lib/languages/json";
  import markdown from "highlight.js/lib/languages/markdown";
  import python from "highlight.js/lib/languages/python";
  import rust from "highlight.js/lib/languages/rust";
  import shell from "highlight.js/lib/languages/shell";
  import typescript from "highlight.js/lib/languages/typescript";
  import "highlight.js/styles/github-dark.css";

  interface Props {
    content: string;
  }

  let { content }: Props = $props();

  hljs.registerLanguage("bash", bash);
  hljs.registerLanguage("css", css);
  hljs.registerLanguage("javascript", javascript);
  hljs.registerLanguage("js", javascript);
  hljs.registerLanguage("json", json);
  hljs.registerLanguage("markdown", markdown);
  hljs.registerLanguage("md", markdown);
  hljs.registerLanguage("python", python);
  hljs.registerLanguage("py", python);
  hljs.registerLanguage("rust", rust);
  hljs.registerLanguage("rs", rust);
  hljs.registerLanguage("shell", shell);
  hljs.registerLanguage("sh", shell);
  hljs.registerLanguage("typescript", typescript);
  hljs.registerLanguage("ts", typescript);

  let html = $derived(renderMarkdown(content));

  function renderMarkdown(value: string): string {
    const raw = marked.parse(value || "", { gfm: true }) as string;
    const clean = DOMPurify.sanitize(raw);
    if (typeof document === "undefined") return clean;

    const template = document.createElement("template");
    template.innerHTML = clean;
    template.content.querySelectorAll("pre > code").forEach((code) => {
      const text = code.textContent ?? "";
      const languageClass = [...code.classList].find((name) => name.startsWith("language-"));
      const language = languageClass?.replace("language-", "");
      if (language && hljs.getLanguage(language)) {
        code.innerHTML = hljs.highlight(text, { language }).value;
      } else {
        code.innerHTML = hljs.highlightAuto(text).value;
      }

      const pre = code.parentElement;
      if (!pre) return;
      const shell = document.createElement("div");
      shell.className = "code-shell";
      const copy = document.createElement("button");
      copy.type = "button";
      copy.className = "code-copy";
      copy.textContent = "Copy";
      pre.replaceWith(shell);
      shell.append(copy, pre);
    });
    return template.innerHTML;
  }

  function handleClick(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (!target.classList.contains("code-copy")) return;
    const shell = target.closest(".code-shell");
    const code = shell?.querySelector("code");
    const text = code?.textContent;
    if (!text) return;
    navigator.clipboard
      .writeText(text)
      .then(() => {
        target.textContent = "Copied";
        setTimeout(() => {
          target.textContent = "Copy";
        }, 900);
      })
      .catch(console.error);
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="markdown" onclick={handleClick}>
  {@html html}
</div>

<style>
  .markdown {
    color: #d9d4c7;
    font-size: 0.78rem;
    line-height: 1.55;
    overflow-wrap: anywhere;
  }

  .markdown :global(p) {
    margin: 0.25rem 0 0.6rem;
  }

  .markdown :global(p:last-child) {
    margin-bottom: 0;
  }

  .markdown :global(a) {
    color: #60a5fa;
  }

  .markdown :global(ul),
  .markdown :global(ol) {
    padding-left: 1.15rem;
    margin: 0.35rem 0 0.65rem;
  }

  .markdown :global(li + li) {
    margin-top: 0.2rem;
  }

  .markdown :global(code:not(pre code)) {
    background: #0e0e0e;
    border: 1px solid #303030;
    border-radius: 4px;
    color: #f5f5f5;
    font-family: "SF Mono", Menlo, Monaco, monospace;
    font-size: 0.72rem;
    padding: 0.05rem 0.25rem;
  }

  .markdown :global(.code-shell) {
    position: relative;
    margin: 0.55rem 0;
    border: 1px solid #303030;
    border-radius: 6px;
    background: #0d0d0d;
    overflow: hidden;
  }

  .markdown :global(pre) {
    margin: 0;
    padding: 0.75rem;
    overflow-x: auto;
  }

  .markdown :global(pre code) {
    font-family: "SF Mono", Menlo, Monaco, monospace;
    font-size: 0.72rem;
  }

  .markdown :global(.code-copy) {
    position: absolute;
    top: 0.35rem;
    right: 0.35rem;
    background: #1f1f1f;
    border: 1px solid #3a3a3a;
    border-radius: 4px;
    color: #b8b8b8;
    cursor: pointer;
    font-family: inherit;
    font-size: 0.65rem;
    padding: 0.15rem 0.4rem;
  }

  .markdown :global(.code-copy:hover) {
    color: #f5f5f5;
    border-color: #555;
  }
</style>
