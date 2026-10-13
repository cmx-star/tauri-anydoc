import { createApp } from "vue";
import App from "./App.vue";
import "./style.css";
import "highlight.js/styles/atom-one-dark.css";
import { marked, Renderer } from "marked";
import hljs from "highlight.js";

const renderer = new Renderer();
renderer.code = function ({ text, lang }: { text: string; lang?: string }) {
  const validLang = lang && hljs.getLanguage(lang) ? lang : "plaintext";
  const highlighted = hljs.highlight(text, { language: validLang }).value;
  return `<pre><code class="hljs language-${validLang}">${highlighted}</code></pre>`;
};

marked.use({ renderer, gfm: true, breaks: true });

createApp(App).mount("#app");
