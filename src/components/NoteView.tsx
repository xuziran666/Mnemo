import ReactMarkdown, { type Components } from "react-markdown";
import remarkGfm from "remark-gfm";
import remarkMath from "remark-math";
import rehypeKatex from "rehype-katex";
import rehypeHighlight from "rehype-highlight";
import { openUrl } from "@tauri-apps/plugin-opener";
import "katex/dist/katex.min.css";
// 必须用暗色主题：全站是 #1e1e1e/#2a2a2a 深色，配浅色 github.css 会导致代码块几乎不可读。
import "highlight.js/styles/github-dark.css";

// react-markdown 默认的 urlTransform 会过滤掉 javascript: 之类的协议，
// 所以这里只需要处理「点击后 WebView 原地导航」的问题。
const EXTERNAL_URL = /^(https?:|mailto:|tel:)/i;

// 自定义链接渲染：点击时必须 preventDefault。
// 否则笔记里的链接会让 WebView 直接导航到外站，Mnemo 窗口变成一个浏览器页面且无法回到应用。
// 绝对地址交给 opener 插件用系统默认程序打开；相对地址在 Tauri 里没有意义，直接忽略。
const components: Components = {
  a({ href, title, children }) {
    return (
      <a
        href={href}
        title={title}
        onClick={(e) => {
          e.preventDefault();
          if (href && EXTERNAL_URL.test(href)) {
            void openUrl(href).catch(() => undefined);
          }
        }}
      >
        {children}
      </a>
    );
  },
};

// NoteView 负责把 Markdown 内容渲染成富文本阅读界面。
// 这里组合了 GFM、数学公式、代码高亮插件，使 Note 类型不仅能显示文字，还能按真实文档方式展示表格、公式和代码块。
export default function NoteView({ content }: { content: string }) {
  return (
    <div className="markdown-body">
      <ReactMarkdown
        remarkPlugins={[remarkGfm, remarkMath]}
        rehypePlugins={[rehypeKatex, rehypeHighlight]}
        components={components}
      >
        {content}
      </ReactMarkdown>
    </div>
  );
}
