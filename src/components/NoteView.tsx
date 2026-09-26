import ReactMarkdown, { type Components } from "react-markdown";
import { useEffect } from "react";
import remarkGfm from "remark-gfm";
import remarkMath from "remark-math";
import rehypeKatex from "rehype-katex";
import rehypeHighlight from "rehype-highlight";
import { openUrl } from "@tauri-apps/plugin-opener";
import "katex/dist/katex.min.css";
import hljsDark from "highlight.js/styles/github-dark.css?url";
import hljsLight from "highlight.js/styles/github.css?url";
import { onThemeChange, readTheme, type Theme } from "../theme";

// 两套官方主题都用同特异性的裸 .hljs-* 选择器（0,1,0），
// 无法靠外层包裹区分，所以只能整体替换 stylesheet：
// 这里用 ?url 拿到资源地址，再改 index.html 里那个 <link id="hljs-theme"> 的 href。
// katex.min.css 无需处理：它零硬编码颜色，全部继承 currentColor，会自动跟随主题。
const HLJS_HREF: Record<Theme, string> = { dark: hljsDark, light: hljsLight };

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
  // 挂载时必须先应用一次：index.html 里的 <link id="hljs-theme"> 初始没有 href，
  // 不补这一次的话首次打开的代码块会是无样式的。
  // link 元素在 <head> 里常驻，所以每次切主题只改 href，不会有样式表累积。
  useEffect(() => {
    const apply = (t: Theme) => {
      const link = document.getElementById("hljs-theme") as HTMLLinkElement | null;
      if (link) link.href = HLJS_HREF[t];
    };
    apply(readTheme());
    return onThemeChange(apply);
  }, []);

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
