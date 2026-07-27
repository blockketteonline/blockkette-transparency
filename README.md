# Blockkette – Non‑Custodial Super App (Wallet + AI Advisor)

**A fully non‑custodial wallet for 31+ blockchains, integrated with a 4H‑timeframe trading intelligence engine – frontend + backend in one repository.**

---

## 🧩 Repository Structure blockkette-transparency/
├── README.md
├── frontend/
│ └── src/
│ ├── hooks/
│ │ └── useWallet.tsx ← Wallet core (create, unlock, recover)
│ └── pages/
│ └── AIAdvisorPanel.tsx ← Live 4H analysis panel
└── backend/
└── app/
└── routers/
└── ai_advisor.py ← AI engine (strategies, indicators, LLM) 
*(Additional modules – Perps, Factory, Genesis DEX – will be added in subsequent commits.)*

---

## 🧠 AI Trading Advisor

### What it does
- Pulls **live 4H OHLCV data** from 6 independent sources (Binance, Kraken, CoinGecko, …) with automatic fallback.
- Computes **technical indicators from scratch** (EMA, RSI, ATR, volatility) – pure Python, auditable, no third‑party TA libraries.
- Runs **three strategies** simultaneously:
  - Trend‑Following (EMA crossover + pullback detection)
  - Breakout Trading (volume‑confirmed range breaks)
  - Opening Range Breakout (ORB)
- Validates signals with **LLMs (DeepSeek, Gemini, GLM)** but enforces a **hard 2:1 reward:risk rule** – the AI cannot override your risk settings.
- Returns concrete **long/short/wait** decisions **with exact position sizing** (margin‑aware, leverage‑adjusted).

### Frontend Integration
The `AIAdvisorPanel.tsx` React component:
- Displays price, RSI, SMA, ATR, 30‑day range
- Auto‑fetches a fresh 4H analysis on every asset switch
- “Set up as Long / Short” buttons feed signals directly to the Perps order form

### API Endpoints
| Endpoint | Purpose | Consumer |
|----------|---------|----------|
| `POST /api/ai/analyze` | Indicators + AI reasoning | `AIAdvisorPanel.tsx` |
| `POST /api/ai/decide`  | Sized trading decision | Perps order form, automation |
| `POST /api/ai/chat`    | Conversational portfolio advice | Chat panel |
| `GET  /api/ai/health`  | Provider status | DevOps |

### Live Test
```bash
curl -X POST https://blockkette.online/api/ai/analyze \
  -H "Content-Type: application/json" \
  -d '{"market":"BTC-PERP","account_margin_usd":1000,"risk_tolerance":"moderate"}'
📐 The Mathematics (every indicator is computed natively)

Exponential Moving Average (EMA)

E
M
A
t
=
P
r
i
c
e
t
×
k
+
E
M
A
t
−
1
×
(
1
−
k
)
,
k
=
2
p
e
r
i
o
d
+
1
EMA 
t
​	
 =Price 
t
​	
 ×k+EMA 
t−1
​	
 ×(1−k),k= 
period+1
2
​	
 
Seeded with a simple average. Used for EMA20, EMA50, EMA200.

Relative Strength Index (RSI)

R
S
=
Average Gain
Average Loss
over 14 periods
RS= 
Average Loss
Average Gain
​	
 over 14 periods
R
S
I
=
100
−
100
1
+
R
S
RSI=100− 
1+RS
100
​	
 
Overbought ≥ 70, oversold ≤ 30.

Average True Range (ATR)

T
R
=
max
⁡
(
H
i
g
h
−
L
o
w
,
∣
H
i
g
h
−
P
r
e
v
i
o
u
s
C
l
o
s
e
∣
,
∣
L
o
w
−
P
r
e
v
i
o
u
s
C
l
o
s
e
∣
)
TR=max(High−Low,∣High−PreviousClose∣,∣Low−PreviousClose∣)
A
T
R
t
=
13
×
A
T
R
t
−
1
+
T
R
t
14
ATR 
t
​	
 = 
14
13×ATR 
t−1
​	
 +TR 
t
​	
 
​	
 
Used for stop‑loss placement.

Slope (Linear Regression over N bars)

S
l
o
p
e
=
∑
(
i
−
x
ˉ
)
(
y
i
−
y
ˉ
)
∑
(
i
−
x
ˉ
)
2
Slope= 
∑(i− 
x
ˉ
 ) 
2
 
∑(i− 
x
ˉ
 )(y 
i
​	
 − 
y
ˉ
​	
 )
​	
 
Detects trend strength on EMA series.

Daily Volatility (σ)

r
t
=
ln
⁡
(
C
l
o
s
e
t
C
l
o
s
e
t
−
1
)
,
σ
=
std
(
r
)
×
periods per day
r 
t
​	
 =ln( 
Close 
t−1
​	
 
Close 
t
​	
 
​	
 ),σ=std(r)× 
periods per day
​	
 
Reported as a percentage.

🤖 Machine‑Learning Roadmap (post‑YC)

We will replace generic LLMs with a custom ML pipeline:

Fine‑tuned transformers trained on order‑book, perps liquidations, on‑chain whale movements.
Sentiment analysis from X, Discord, Telegram.
Cross‑market pattern recognition (Graph Neural Networks).
Reinforcement learning for adaptive backtesting.
Kalman‑filter trend estimation instead of static EMAs – already prototyped.
The resulting models will be token‑gated (see Subscription Model).

💎 Subscription Model – Powered by Genesis Token

Free tier – Basic indicators, 1 analysis per hour.
Pro tier – Real‑time signals, all three strategies, unlimited calls – unlocked by staking GNS tokens.
Enterprise tier – Custom models trained on your own history, on‑chain, paid in GNS.
This creates a recurring revenue stream for the protocol.

🔐 Non‑Custodial Wallet Security

1. Keys never leave the browser

Wallet creation (useWallet.tsx) uses generateWalletsFromSeed to derive keys for 31+ chains directly in the user’s browser.
The seed phrase is never transmitted to any server.
2. Local encryption (AES‑256‑GCM)

The seed + all derived private keys are encrypted with AES‑256‑GCM, using a key derived from the user’s password via PBKDF2 (100,000 iterations).
The encrypted blob is stored in IndexedDB – never sent over the network.
3. Zero‑knowledge architecture

The password itself is never stored. Only the encrypted data is persisted.
The backend receives only public information (market, margin, transaction details). It has no access to private keys, seed phrases, or passwords.
Transaction signing happens exclusively in the browser. The backend cannot sign anything on behalf of the user.
4. Auditable from the browser (verify yourself)

Open the Blockkette wallet on https://blockkette.online, press F12 to open Developer Tools, and:

Network tab: Create a wallet or unlock it – you will see zero network requests containing the password or seed phrase.
Application tab → IndexedDB: The encrypted wallet blob is stored locally and is unintelligible without the correct password.
Source code: Every encryption and key‑derivation function is open‑source in this repository (see frontend/src/utils/crypto.tsx).
🔒 Why Blockkette Cannot Access User Funds

We don’t have the seed phrase – generated locally and encrypted.
We don’t have the password – never sent anywhere.
We can’t sign transactions – private keys are decrypted in‑memory only for signing and discarded.
Backend is stateless – no user wallets, no encrypted blobs.
Even if our servers were compromised, there would be nothing to steal.

🌐 Live Demo

https://blockkette.online

Create a testnet wallet (fully in‑browser)
Open the AI Trade Analysis panel to see live 4H signals
Inspect network traffic to confirm no private data leaves your machine
Built by the Blockkette team. Not financial advice. Testnet only.
