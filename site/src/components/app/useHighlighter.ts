/**
 * Loads the syntax highlighter when some code wants colouring, and re-renders
 * the caller once it has arrived.
 */
import { useEffect, useSyncExternalStore } from "react";
import { isHighlighterReady, loadHighlighter, subscribeHighlighter } from "../../lib/codeHighlight";

export function useHighlighter(wanted: boolean): boolean {
  useEffect(() => {
    if (wanted) void loadHighlighter();
  }, [wanted]);
  return useSyncExternalStore(subscribeHighlighter, isHighlighterReady, () => false);
}
