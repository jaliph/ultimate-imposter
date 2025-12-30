# Agent-X: Social Deduction Game

A fun social deduction game built with Dioxus 0.7 where players try to find the imposter among them!

## 🎮 Game Overview

**Agent-X** is a mobile-friendly web game for 3+ players. One player is randomly assigned as the imposter with a different word, while all other players receive the same word. Through discussion and voting, players try to identify the imposter before they blend in successfully.

## 🚀 Quick Start

### Prerequisites
- Rust installed on your system
- Dioxus CLI

### Installation

1. Install Dioxus CLI:
```bash
curl -sSL http://dioxus.dev/install.sh | sh
```

2. Run the game locally:
```bash
dx serve --platform web
```

The game will be available at `http://localhost:8080`

### 📱 Running on Mobile (Same Network)

To access the game from your mobile phone on the same WiFi network:

1. The app is already configured to bind to all network interfaces (`0.0.0.0:8080`)
2. Find your computer's local IP address:
   - **macOS/Linux**: `ifconfig | grep "inet " | grep -v 127.0.0.1`
   - **Windows**: `ipconfig`
3. On your mobile device, open the browser and navigate to:
   ```
   http://YOUR_COMPUTER_IP:8080
   ```
   Example: `http://192.168.1.100:8080`

This allows all players to use the same device or different devices on the network!

## 📱 How to Play

1. **Setup**: Enter the number of players (3+) and their names
2. **Card View**: Each player reveals their card privately (pass the device around)
3. **Discussion**: Players discuss to find who has the odd word
4. **Voting**: Vote for who you think is the imposter
5. **Scoring**: 
   - If imposter found: Regular players get 10 points each
   - If imposter not found: Imposter gets 20 points

## 📂 Project Structure

```
agent-x/
├─ assets/
│  ├─ favicon.ico
│  └─ styling/
│     └─ game.css      # Game styling
├─ src/
│  ├─ main.rs          # App entry point and routing
│  └─ views/
│     ├─ mod.rs        # Views module
│     └─ game.rs       # Game logic and components
└─ Cargo.toml
```

## 🎨 Features

- Beautiful gradient UI with smooth animations
- Mobile-optimized responsive design
- Privacy-focused card reveal system
- Score tracking across multiple rounds
- Supports 3-10 players

## 🛠️ Technology

- **Framework**: Dioxus 0.7
- **Language**: Rust
- **Styling**: Custom CSS

## 📄 License

MIT License
