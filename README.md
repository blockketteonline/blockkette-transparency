# Blockkette AI Advisor (Full‑Stack)

**Non‑custodial trading intelligence powering the wallet and Perps terminal – frontend + backend in one repository.**

---

## 🧩 Repository structure


---

## 🧠 End‑to‑end architecture

### 1. Backend (`ai_advisor.py`)
- Pulls **live 4H OHLCV data** from 6 independent sources (Binance, Kraken, CoinGecko, …) with automatic fallback.
- Computes **technical indicators** (EMA, RSI, ATR, volatility, trend structure) **from scratch** – pure Python, auditable, zero dependencies.
- Runs **three battle‑tested strategies** simultaneously:
  - Trend‑Following (EMA crossover + pullback detection)
  - Breakout Trading (volume‑confirmed range breaks)
  - Opening Range Breakout (ORB)
- Validates signals with **LLMs (DeepSeek, Gemini, GLM)** while enforcing a **hard 2:1 reward:risk rule** – the AI cannot override your risk.
- Returns concrete **long/short/wait** decisions **with exact position sizing**.

### 2. Frontend (`AIAdvisorPanel.tsx`)
- User picks **market, risk tolerance, and margin** directly inside the wallet UI.
- Auto‑fetches a fresh 4H analysis on every asset switch.
- Displays **price, RSI, SMA, ATR, 30‑day range**, and the full AI reasoning.
- Buttons “Set up as Long / Short” forward the signal to the **Perps order form** – one tap to execute.

### 3. Non‑custodial wallet integration
- The panel lives inside the wallet dashboard.
- It calls `POST /api/ai/analyze` with **non‑sensitive** account parameters.
- The AI responds with a trading idea – **you sign the transaction locally** with your own keys.
- For executable orders, `POST /api/ai/decide` returns a fully‑sized position locked to 2:1 R:R.

---

## 📡 API endpoints

| Endpoint | Purpose | Consumer |
|----------|---------|----------|
| `POST /api/ai/analyze` | Indicators + AI reasoning (quick read) | `AIAdvisorPanel.tsx` |
| `POST /api/ai/decide`  | Tradable decision with exact sizing | Perps order form, automated strategies |
| `POST /api/ai/chat`    | Conversational AI for portfolio questions | AI chat panel |
| `GET  /api/ai/health`  | Provider status check | DevOps / monitoring |

---

## 📐 The maths behind every indicator

All technical indicators are computed **natively** – no third‑party TA libraries.  
This makes the code auditable, portable, and ready for on‑chain verification.

### Exponential Moving Average (EMA)
Used for EMA20, EMA50, EMA200. The first value is seeded with a simple average.

### Relative Strength Index (RSI)
Measures volatility – used for stop‑loss placement.

### Slope (Linear Regression over N bars)
Detects trend strength on EMA series.

### Daily Volatility (σ)
Reported as a percentage.

---

## 🤖 Machine‑Learning research – evolving beyond indicators

We are not stopping at classical TA. Our team is developing an **advanced ML research pipeline** to turn this module into a self‑improving trading brain.

**Immediate post‑YC goals:**
1. **Fine‑tuned transformers** trained on:
   - 4H order‑book snapshots from 100+ blockchains
   - Perps liquidations, funding rates, open interest
   - On‑chain wallet behaviour (whale moves, exchange inflows)

2. **Sentiment analysis** ingesting X, Discord, Telegram, and on‑chain metrics – converting unstructured text into quantifiable features.

3. **Cross‑market pattern recognition** using graph neural networks – the AI will learn that a BTC options expiry impacts SOL liquidity 12h later.

4. **Reinforcement learning agent** that backtests strategies across years of data and continuously adapts to new market regimes.

**Math upgrade example – from static EMA to adaptive Kalman‑filter trend**
This dynamically adapts to volatility – faster than any fixed‑window EMA. Our research is actively integrating such models for the Pro and Enterprise tiers.

---

## 💎 Subscription model – powered by Genesis Token

This AI becomes a **token‑gated premium service**:

- **Free tier** – basic indicators, 1 analysis per hour.
- **Pro tier** – real‑time 4H signals, all three strategies, unlimited calls – **unlocked by staking GNS tokens**.
- **Enterprise tier** – custom ML models trained on your own trading history, running on‑chain, paid for in GNS.

Staking GNS grants access to the most advanced models, creating a **recurring revenue stream** for the protocol.

---

## 🔒 Security & Auditing

- All indicators and risk‑enforcement are **deterministic** – they run locally, zero external calls.
- **Hard‑coded 2:1 R:R rule** – no AI hallucination can override it.
- The frontend never touches private keys; it only calls public API endpoints with non‑sensitive account data.
- Codebase is being prepared for a **professional institutional audit** (Trail of Bits / OtterSec) after YC funding.

---

## 🌐 Live test

```bash
curl -X POST https://blockkette.online/api/ai/analyze \
  -H "Content-Type: application/json" \
  -d '{"market":"BTC-PERP","account_margin_usd":1000,"risk_tolerance":"moderate"}'
cd backend
pip install -r requirements.txt
uvicorn app.main:app --reload
cd frontend
npm install
npm run dev
Built by the Blockkette team. Not financial advice. Testnet only.


