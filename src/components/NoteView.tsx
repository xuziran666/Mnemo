import { useMemo } from "react";
import MarkdownIt from "markdown-it";

const md = new MarkdownIt({ html: false });

export default function NoteView({ content }: { content: string }) {
  const html = useMemo(() => md.render(content), [content]);
  return <div className="markdown-body" dangerouslySetInnerHTML={{ __html: html }} />;
}
