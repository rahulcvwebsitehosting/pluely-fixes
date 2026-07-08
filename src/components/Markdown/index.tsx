import React from "react";
import { Streamdown } from "streamdown";
import "katex/dist/katex.min.css";
import { openUrl } from "@tauri-apps/plugin-opener";

interface MarkdownRendererProps {
  children: string;
  isStreaming?: boolean;
}

const THINK_TAG_RE = /^(\s*)(<think>)([\s\S]*?)(<\/think>)(\s*)$/im;
const THINK_TAG_RE_GLOBAL = /<think>([\s\S]*?)<\/think>/g;

export function Markdown({
  children,
  isStreaming = false,
}: MarkdownRendererProps) {
  // During streaming, leave think tags raw so Streamdown renders them as
  // plain text (avoids layout thrash).  Once done, render properly.
  const processed = isStreaming
    ? children
    : renderThinkTags(children);

  return (
    <Streamdown
      isAnimating={isStreaming}
      shikiTheme={["github-light", "github-dark"]}
      components={COMPONENTS as any}
      controls={{
        table: true,
        code: true,
        mermaid: {
          download: true,
          copy: true,
          fullscreen: false,
          panZoom: false,
        },
      }}
    >
      {processed}
    </Streamdown>
  );
}

function renderThinkTags(text: string): string {
  if (!THINK_TAG_RE.test(text)) return text;

  return text.replace(THINK_TAG_RE_GLOBAL, (_match, content) => {
    const label = "Reasoning";
    return `\n\n<details class="think-block">\n<summary>${label}</summary>\n\n${content.trim()}\n\n</details>\n\n`;
  });
}

const COMPONENTS = {
  a: ({ children, href, ...props }: any) => {
    const handleClick = async (e: React.MouseEvent) => {
      e.preventDefault();
      if (href) {
        try {
          await openUrl(href);
        } catch (error) {
          console.error("Failed to open URL:", error);
        }
      }
    };

    return (
      <a
        href={href}
        className="text-gray-600 underline underline-offset-2 hover:text-gray-800 dark:text-gray-300 dark:hover:text-gray-100 cursor-pointer"
        onClick={handleClick}
        {...props}
      >
        {children}
      </a>
    );
  },
};
