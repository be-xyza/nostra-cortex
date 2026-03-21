import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { VitePWA } from "vite-plugin-pwa";

function resolveGatewayTargets() {
  const httpTarget = process.env.CORTEX_WEB_GATEWAY_URL || process.env.VITE_CORTEX_GATEWAY_URL || "http://127.0.0.1:3000";
  const wsTarget = httpTarget.replace(/^http/i, "ws");
  return { httpTarget, wsTarget };
}

export default defineConfig(() => {
  const { httpTarget, wsTarget } = resolveGatewayTargets();

  return {
    plugins: [
      react(),
      VitePWA({
        strategies: 'injectManifest',
        srcDir: 'src',
        filename: 'sw.ts',
        injectRegister: 'auto',
        manifest: {
          name: "Nostra",
          short_name: "Nostra",
          display: "standalone",
          start_url: "/",
          background_color: "#0b0b0f",
          theme_color: "#0b0b0f",
          icons: [
            {
              src: "/icon-192.png",
              sizes: "192x192",
              type: "image/png"
            },
            {
              src: "/icon-512.png",
              sizes: "512x512",
              type: "image/png",
              purpose: "any maskable"
            }
          ]
        },
        devOptions: {
          enabled: true,
          type: 'module',
        },
        workbox: {
          maximumFileSizeToCacheInBytes: 5000000,
        },
        injectManifest: {
          maximumFileSizeToCacheInBytes: 5000000,
        }
      })
    ],
    server: {
      port: 4173,
      strictPort: true,
      host: '127.0.0.1',
      proxy: {
        "/api": {
          target: httpTarget,
          changeOrigin: true
        },
        "/ws": {
          target: wsTarget,
          ws: true,
          changeOrigin: true
        }
      }
    },
    preview: {
      port: 4173,
      strictPort: true,
      host: '127.0.0.1',
      proxy: {
        "/api": {
          target: httpTarget,
          changeOrigin: true
        },
        "/ws": {
          target: wsTarget,
          ws: true,
          changeOrigin: true
        }
      }
    }
  };
});
