import { useEffect, useRef, type Ref, type TextareaHTMLAttributes } from "react";

interface Props extends TextareaHTMLAttributes<HTMLTextAreaElement> {
  maxHeight?: number;
  ref?: Ref<HTMLTextAreaElement>;
}

export default function AutoGrowTextarea({ value, maxHeight = 240, ref, ...rest }: Props) {
  const innerRef = useRef<HTMLTextAreaElement>(null);

  function setRefs(el: HTMLTextAreaElement | null) {
    innerRef.current = el;
    if (typeof ref === "function") ref(el);
    else if (ref) ref.current = el;
  }

  useEffect(() => {
    const el = innerRef.current;
    if (!el) return;
    el.style.height = "auto";
    const height = Math.min(el.scrollHeight, maxHeight);
    el.style.height = `${height}px`;
    el.style.overflowY = el.scrollHeight > maxHeight ? "auto" : "hidden";
  }, [value, maxHeight]);

  return <textarea ref={setRefs} value={value} {...rest} />;
}
