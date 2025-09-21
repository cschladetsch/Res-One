# Resonant - Technical Design v0.1

## Core Concept
Daily 4D fractal generation + mathematical social interactions. Zero input required, purely ambient social presence through geometric beauty.

## Architecture

### Client (Web-first, WASM)
- **Rust → WASM** fractal engine (3D Mandelbulb ray marching)
- **WebGL 2.0** rendering (60fps on 2018+ phones)
- **Web Audio API** for fractal→music synthesis
- **PWA** for all platforms (iOS/Android via web, no app stores)

### Backend (Minimal, Rust)
- **Seed generation service** (stateless, user_id + timestamp)
- **Transform relay** (WebSocket for real-time echoes)
- **URL shortener** for sharing (time-limited tokens)
- **Redis cache** for active transforms

### Data Flow
```
User wake → Generate seed → Render fractal →
Gesture transforms → Relay to peers → Update fractal
```

### Viral Mechanics
1. **Daily fractal** auto-generates on wake/app open
2. **Gesture transforms** sent to friend circle (swipe/tilt/smile)
3. **Freeze mechanism** - save one fractal per day as "champion"
4. **Battle system** - new fractal vs frozen champion (frequency interference)
5. **Share URLs** - anyone can view, gesture back creates account seed

## Social Model

### Groups & Resonance
- Multiple circles (family/friends/work) contribute different frequency bands
- **Atemporal interaction** - no timestamps, no pressure
- **Pass-through transforms** - forwarding unchanged is valid participation
- **Network effects** - late risers get more complex fractals

### Privacy & Security

#### Proximity Features
**DISABLED BY DEFAULT** - physical proximity detection creates security risks:
- Location tracking vectors
- Unwanted social graphs
- Stalking potential

**Alternative:** Manual URL sharing via existing secure channels (Signal, iMessage, etc.)

#### Biometric Input (Optional)
- **On-device only** - face/movement data never leaves device
- Only mathematical transforms transmitted
- User controls what influences fractal

#### URL Security
- **Time-limited tokens** (24-48 hour expiry)
- **Non-guessable seeds** (cryptographic random)
- **No persistent user tracking** in shared URLs

## Technical Stack

### Dependencies
```toml
wasm-bindgen = "0.2"    # JS interop
nalgebra = "0.32"       # Math/linear algebra
web-sys = "0.3"         # Browser APIs
```

### Scaling Path
1. **MVP**: Single VPS + CDN (0-10K users)
2. **Growth**: Auto-scaling groups + Redis (10K-100K)
3. **Viral**: Edge computing (Cloudflare Workers) for transform relay
4. **P2P transition**: WebRTC data channels, servers only for discovery

### Performance Targets
- **< 200KB** initial load (WASM + JS)
- **< 16ms** per frame (60fps)
- **< 1KB** per interaction (transform data)
- **< 50ms** transform relay latency

## Key Differentiators
- **Zero user input** - fractals generate automatically
- **Mathematical communication** - no text/emoji, pure geometry
- **Ambient social presence** - like lava lamp that responds to friends
- **Language-agnostic** - works across all cultures
- **Time-flexible** - async but rewards sync interactions

## Viral Validation Metrics
- **Daily return rate** (fractal curiosity)
- **Share conversion** (recipient creates account)
- **Group formation** (friend circles establish)
- **Complexity accumulation** (engagement drives beauty)