#!/bin/bash
set -e
echo "=== Building frontend ==="
cd frontend
npm run build
echo "=== Deploying to server ==="
ssh root@178.105.168.55 "cd /root/bridge_club && \
  git pull origin main && \
  cd frontend && npm install && npm run build && \
  rm -rf /var/www/bridge-club && cp -r build /var/www/bridge-club && \
  cd /root/bridge_club/backend && source \"\$HOME/.cargo/env\" && cargo build --release -p bridge-server && \
  systemctl restart bridge-server"
echo "=== Done ==="
