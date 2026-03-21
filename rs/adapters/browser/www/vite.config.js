import { defineConfig } from 'vite';
import fs from 'fs';

export default defineConfig({
  server: {
    port: 4321,
    host: '0.0.0.0',
    https: {
      key: fs.readFileSync('/tmp/key.pem'),
      cert: fs.readFileSync('/tmp/cert.pem'),
    },
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
  },
});
