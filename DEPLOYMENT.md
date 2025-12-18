# DEPLOYMENT GUIDE: Chrono-Hypernova

This guide explains how to deploy the bot to a headless cloud environment (AWS, DigitalOcean, etc.) with 24/7 uptime.

## Recommended Server Environment
- **Provider**: AWS (EC2), DigitalOcean (Droplet), or Hetzner.
- **Instance Size**: `t3.medium` (2 vCPUs, 4GB RAM) for low latency.
- **Region**: `us-east-1` (Virginia) is recommended for proximity to major API endpoints.
- **OS**: Ubuntu 22.04 LTS.

## Server Setup Commands

Run these commands on a fresh server to install dependencies:

```bash
# 1. Update system
sudo apt update && sudo apt upgrade -y

# 2. Install Build Essentials & SSL
sudo apt install build-essential pkg-config libssl-dev -y

# 3. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 4. Clone & Build
git clone https://github.com/your-repo/chrono-hypernova.git
cd chrono-hypernova
cargo build --release
```

## 24/7 Process Management (Systemd)

To ensure the bot restarts automatically if it crashes or the server reboots:

1. Create a service file:
   `sudo nano /etc/systemd/system/polyarb.service`

2. Paste the following configuration:
   ```ini
   [Unit]
   Description=PolyArb Chrono-Hypernova Bot
   After=network.target

   [Service]
   Type=simple
   User=ubuntu
   WorkingDirectory=/home/ubuntu/chrono-hypernova
   EnvironmentFile=/home/ubuntu/chrono-hypernova/.env
   ExecStart=/home/ubuntu/chrono-hypernova/target/release/polyarb
   Restart=always
   RestartSec=5

   [Install]
   WantedBy=multi-user.target
   ```

3. Enable and Start:
   ```bash
   sudo systemctl daemon-reload
   sudo systemctl enable polyarb
   sudo systemctl start polyarb
   sudo systemctl status polyarb
   ```

## Docker Alternative
If you prefer Docker, use the included `Dockerfile` (to be added) and run:
`docker-compose up -d`

## Accessing the Web Dashboard
Once running, the bot will host a GUI at `http://your-server-ip:3000`. 
Ensure port `3000` is open in your server's Security Group/Firewall.
