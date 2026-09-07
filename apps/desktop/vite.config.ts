import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
	plugins: [react()],
	clearScreen: false,
	build: {
		chunkSizeWarningLimit: 600,
		rollupOptions: {
			output: {
				// NOTE: only app-level vendor code is split out. Splitting
				// @vidstack/react too creates a circular chunk
				// (vidstack <-> tanstack) and fights its own dynamic imports.
				manualChunks: {
					tanstack: [
						"@tanstack/react-query",
						"@tanstack/react-router",
						"@tanstack/react-virtual",
					],
				},
			},
		},
	},
	server: {
		port: 1420,
		strictPort: true,
		host: host || false,
		hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
		watch: { ignored: ["**/src-tauri/**"] },
	},
});
