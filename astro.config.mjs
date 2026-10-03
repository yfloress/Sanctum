import { defineConfig } from 'astro/config';
import react from '@astrojs/react';
import AstroPWA from '@vite-pwa/astro';
import tailwindcss from '@tailwindcss/vite';
import { fileURLToPath } from 'node:url';

const site = 'https://yfloress.github.io';
const base = '/Sanctum/';

export default defineConfig({
  site,
  base,
  output: 'static',
  trailingSlash: 'always',

  integrations: [
    react(),
    AstroPWA({
      registerType: 'autoUpdate',
      manifest: {
        name: 'Sanctum Web',
        short_name: 'Sanctum',
        description: 'Your Personal Finance Fortress - Offline JSON Generator',
        theme_color: '#0b1220',
        background_color: '#0b1220',
        display: 'standalone',
        start_url: base,
        scope: base,
        icons: [
          {
            src: 'pwa.svg',
            sizes: 'any',
            type: 'image/svg+xml',
            purpose: 'any',
          },
        ],
      },
      workbox: {
        globPatterns: ['**/*.{js,css,html,svg,png,woff2}'],
        // Bunny serves the fonts; cache them so an offline visit keeps its type.
        runtimeCaching: [
          {
            urlPattern: /^https:\/\/fonts\.bunny\.net\/.*/i,
            handler: 'CacheFirst',
            options: {
              cacheName: 'bunny-fonts',
              expiration: { maxEntries: 12, maxAgeSeconds: 60 * 60 * 24 * 365 },
              cacheableResponse: { statuses: [0, 200] },
            },
          },
        ],
        navigateFallback: `${base}404.html`,
      },
    }),
  ],

  vite: {
    plugins: [tailwindcss()],
    optimizeDeps: {
      include: ['react-dom/client'],
    },
    resolve: {
      alias: {
        '@': fileURLToPath(new URL('./src', import.meta.url)),
        '@components': fileURLToPath(new URL('./src/components', import.meta.url)),
        '@layouts': fileURLToPath(new URL('./src/layouts', import.meta.url)),
        '@lib': fileURLToPath(new URL('./src/lib', import.meta.url)),
      },
    },
  },
});
