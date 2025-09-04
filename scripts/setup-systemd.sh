#!/bin/bash

# Setup script for poke_me systemd service
# This script creates a properly configured systemd service file

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Poke Me Systemd Service Setup${NC}"
echo "=================================="

# Get current user information
USERNAME=$(whoami)
HOME_DIR=$(echo $HOME)
USER_ID=$(id -u)

# Determine binary path
if command -v poke_me &> /dev/null; then
    BINARY_PATH=$(which poke_me)
    echo -e "${GREEN}✓${NC} Found poke_me binary at: $BINARY_PATH"
else
    echo -e "${RED}✗${NC} poke_me binary not found in PATH"
    echo "Please install poke_me first with: cargo install --path ."
    exit 1
fi

# Create the service file from template
SERVICE_FILE="/etc/systemd/system/poke-me.service"
TEMPLATE_FILE="poke-me.service.template"

# Check if template exists
if [ ! -f "$TEMPLATE_FILE" ]; then
    echo -e "${RED}✗${NC} Template file '$TEMPLATE_FILE' not found"
    echo "Please make sure you're running this script from the poke_me directory"
    exit 1
fi

echo -e "${YELLOW}Creating service file from template...${NC}"
# Replace placeholders in template and create service file
sed -e "s/USERNAME/$USERNAME/g" \
    -e "s|HOME_DIR|$HOME_DIR|g" \
    -e "s|BINARY_PATH|$BINARY_PATH|g" \
    -e "s/USER_ID/$USER_ID/g" \
    "$TEMPLATE_FILE" | sudo tee "$SERVICE_FILE" > /dev/null

# Create data directory
DATA_DIR="$HOME_DIR/.local/share/poke_me"
echo -e "${YELLOW}Creating data directory...${NC}"
mkdir -p "$DATA_DIR"

# Reload systemd
echo -e "${YELLOW}Reloading systemd configuration...${NC}"
sudo systemctl daemon-reload

# Enable service
echo -e "${YELLOW}Enabling service for auto-start...${NC}"
sudo systemctl enable poke-me.service

# Start service
echo -e "${YELLOW}Starting service...${NC}"
sudo systemctl start poke-me.service


echo -e "${GREEN}✓${NC} Service setup complete!"
echo ""
echo "Service configuration:"
echo "  User: $USERNAME"
echo "  Home: $HOME_DIR"
echo "  Binary: $BINARY_PATH"
echo "  Data directory: $DATA_DIR"
echo "  Service file: $SERVICE_FILE"
echo ""
echo -e "${GREEN}Setup complete!${NC}"
