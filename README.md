# Blockkette AI Advisor

4H‑timeframe trading intelligence engine that powers the **non‑custodial wallet** and **Perps trading terminal** with real‑time signals.

## 🧠 What it does

- Pulls **live market data** from 6 independent sources (Binance, Kraken, CoinGecko, CoinCap, CryptoCompare, CoinMarketCap) — no single point of failure.
- Computes **real technical indicators** (EMA 20/50/200, RSI, ATR, volatility) without third‑party libraries — every number is auditable.
- Runs **three battle‑tested strategies** simultaneously on 4H candles:
  1. **Trend‑Following** (EMA crossover + pullback detection)
  2. **Breakout Trading** (volume‑confirmed range breaks)
  3. **Opening Range Breakout (ORB)**
- Calls **LLMs (DeepSeek, Gemini, GLM)** to validate the signals, but the final decision is always **overridden by a hard 2:1 reward:risk rule** — the AI can't risk more than you allow.
- Returns **concrete long/short/wait** decisions **with exact position sizing** (based on your margin, risk tolerance, and leverage).

## 🔗 How it connects to the wallet & Perps

1. **Wallet (non‑custodial)** – The AI never sees your keys. The frontend (ciphervault) calls `/api/ai/decide` with your account margin and risk settings. The response is a trading idea — you **sign the transaction locally in your wallet**.
2. **Perps module** – The Perps page uses the same AI endpoint to show **live buy/sell signals**. When the AI says “long BTC with 2:1 R:R”, the Perps UI can pre‑fill the order form so you can execute with one click.
3. **Feedback loop** – Trade results are sent back to the AI to track win rates and improve future recommendations.

## 📈 The subscription vision (powered by Genesis token)

- **Free tier** – Basic indicators, one AI call per hour.
- **Pro tier** – Real‑time signals, all three strategies, unlimited AI calls – **unlocked by staking GNS tokens**.
- **Enterprise tier** – Custom ML models trained on your own trading history, running on‑chain, paid for in GNS.

After YC funding, we will:
- Replace generic LLMs with **fine‑tuned transformers** trained on perps order‑book data.
- Add **on‑chain sentiment analysis** (Twitter, Discord, on‑chain metrics).
- Let the AI spot **patterns across 100+ markets** and execute via the wallet’s Perps module.

## 📂 File

The complete source is in `backend/app/routers/ai_advisor.py`.

## 🔒 Security & Auditing

All trading logic is **self‑contained and deterministic**. LLM calls are only for validation. The system enforces a hard 2:1 R:R, regardless of what any model says. We are preparing the codebase for a **full institutional audit** after YC funding.

## 📡 Live Demo

The AI is live on our testnet backend: `https://blockkette.online/api/ai/decide` — you can query it directly (see the code for the POST body format).
