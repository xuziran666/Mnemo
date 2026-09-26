import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { MoonIcon, SunIcon } from "./icons";
import { onThemeChange, readTheme, setTheme, type Theme } from "../theme";

// 工具栏的主题切换按钮。只提供浅色/深色两态，
// 主题初值由 index.html 内联脚本写入，本组件只负责读回和切换。
export function ThemeButton() {
  const { t } = useTranslation();
  const [theme, setCurrent] = useState<Theme>(readTheme);
  useEffect(() => onThemeChange(setCurrent), []);

  const next: Theme = theme === "dark" ? "light" : "dark";

  return (
    <button
      className="add"
      title={`${t(`theme.${theme}`)} → ${t(`theme.${next}`)}`}
      onClick={() => setTheme(next)}
    >
      {theme === "dark" ? <MoonIcon /> : <SunIcon />}
    </button>
  );
}
