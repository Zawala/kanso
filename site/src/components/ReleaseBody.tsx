/**
 * A release's notes as the release notes page shows them: the Markdown
 * GitHub holds, drawn as headings, paragraphs, lists and links. Only web
 * links stay links (see `bodyBlocks`), and text is never inserted as HTML.
 */
import { bodyBlocks } from "../lib/releases";
import type { Block, Inline } from "../lib/markdown";
import { Inlines } from "./app/Preview";

const noWiki = () => undefined;
const text = (c: readonly Inline[]) => <Inlines c={c} onOpenWiki={noWiki} />;

function BlockView({ block }: { block: Block }) {
  switch (block.t) {
    case "heading":
      // The page's own headings are h2 (the version) and h3, so the notes' headings start at h3.
      return block.level <= 2 ? <h3>{text(block.c)}</h3> : <h4 className="font-semibold">{text(block.c)}</h4>;
    case "paragraph":
      return <p>{text(block.c)}</p>;
    case "quote":
      return <blockquote>{text(block.c)}</blockquote>;
    case "hr":
      return <hr />;
    case "code":
      return (
        <pre>
          <code>{block.v}</code>
        </pre>
      );
    case "list": {
      const List = block.ordered ? "ol" : "ul";
      return (
        <List className={block.ordered ? "list-decimal pl-6" : "list-disc pl-6"}>
          {block.items.map((item) => (
            <li key={item.line} style={{ marginLeft: `${item.depth * 1.4}rem` }}>
              {text(item.c)}
            </li>
          ))}
        </List>
      );
    }
    case "table":
      return (
        <div className="overflow-x-auto">
          <table>
            <thead>
              <tr>
                {block.head.map((cell, i) => (
                  <th key={i} className="pr-4 text-left">
                    {text(cell)}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {block.rows.map((row, r) => (
                <tr key={r}>
                  {row.map((cell, i) => (
                    <td key={i} className="pr-4">
                      {text(cell)}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      );
  }
}

export function ReleaseBody({ body }: { body: string }) {
  return (
    <div className="prose-note" data-testid="release-body">
      {bodyBlocks(body).map((block, i) => (
        <BlockView key={i} block={block} />
      ))}
    </div>
  );
}
