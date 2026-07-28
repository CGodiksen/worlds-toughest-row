import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
    plugins: [react()],
    optimizeDeps: {
        exclude: ["maplibre-gl"],
    },
    server: {
        proxy: { "/api": "http://127.0.0.1:4800" },
    },
});
