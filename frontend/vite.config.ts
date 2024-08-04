import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [svelte()],
  base: "/pixelwars/",
  server: {
    hmr: {
      port: 9078
    }
  }
})
