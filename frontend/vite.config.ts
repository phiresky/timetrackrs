import { defineConfig } from 'vite'

export default defineConfig({
  resolve: {
      alias: [
          {
              find: /~(.+)/,
              replacement: "$1"
          }
      ]
  },
  build: {
      target: "esnext"
  },
  browser: {
    'util': false
  }
  

})
