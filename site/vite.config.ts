/**
 * Builds kansoapp.app: TanStack Start rendered on a Cloudflare Worker, with
 * Tailwind compiled at build time. Tests use vitest.config.ts instead, which
 * leaves the Worker runtime out.
 */
import { cloudflare } from "@cloudflare/vite-plugin";
import tailwindcss from "@tailwindcss/vite";
import { tanstackStart } from "@tanstack/react-start/plugin/vite";
import viteReact from "@vitejs/plugin-react";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [cloudflare({ viteEnvironment: { name: "ssr" } }), tailwindcss(), tanstackStart(), viteReact()],
});
