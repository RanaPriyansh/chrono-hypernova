# DEPLOYMENT GUIDE: Chrono-Hypernova

## Cloud Server Requirements
- **Hardware**: AWS EC2 `t3.medium` or DigitalOcean Droplet (2 vCPUs, 4GB RAM).
- **OS**: Ubuntu 22.04 LTS.

## Installation Sequence

```bash
# 1. System Prep
sudo apt update && sudo apt upgrade -y
sudo apt install build-essential pkg-config libssl-dev -y

# 2. Rust Toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 3. Clone & Build
git clone https://github.com/your-repo/chrono-hypernova.git
cd chrono-hypernova
cargo build --release
```

## Systemd Persistence (24/7 Operations)

Create a service file at `/etc/systemd/system/polyarb.service`:

```ini
[Unit]
Description=Chrono-Hypernova PolyArb Bot
After=network.target

[Service]
Type=simple
User=ubuntu
WorkingDirectory=/home/ubuntu/chrono-hypernova
ExecStart=/home/ubuntu/chrono-hypernova/target/release/polyarb
Restart=always
RestartSec=10
# Ensure your .env file is present or use Environment=
EnvironmentFile=/home/ubuntu/chrono-hypernova/.env

[Install]
WantedBy=multi-user.target
```

**Commands:**
```bash
sudo systemctl daemon-reload
sudo systemctl enable polyarb.service
sudo systemctl start polyarb.service
sudo systemctl status polyarb.service
```

## Accessing the Dashboard
The dashboard is served at:
`http://<SERVER_IP>:3000`

Ensure your cloud firewall (Security Group) allows inbound traffic on port 3000.
