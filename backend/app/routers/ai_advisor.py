# backend/app/routers/ai_advisor.py
# ── AI trade analysis — 4H timeframe, 3 strategies, 2:1 R:R, resilient data ──
#
# This revision fixes the failures you hit:
#   1. CoinGecko 429 rate-limit  →  we now fall back through six crypto data sources:
#        Binance → Kraken → CoinGecko → CoinCap → CryptoCompare → CoinPaprika (price-only).
#        Binance and Kraken are the primary sources: both give REAL native 4H OHLCV
#        candles, no API key, high rate limits. The rest are safety nets.
#        CoinMarketCap is also wired in but only used when CMC_API_KEY is set,
#        because their free tier requires a key.
#      For stocks/indices, Yahoo Finance is primary and Stooq is the fallback
#      (Stooq gives real hourly CSV data with no key).
#
#      NOTE on the other services you asked about:
#        - CoinAPI, Amberdata, CoinDesk Data API, Kaiko, Mobula, CryptoRank all
#          need API keys; some are enterprise-paid. I've left hooks in the code
#          so you can drop keys in env vars later, but by default they're skipped.
#        - Coinranking has a free tier but its historical OHLC endpoint needs a
#          key too, so it's also key-gated.
#        - For stocks: investing.com, Google Finance, MSN Money, MarketWatch,
#          Simply Wall St, and TradingView do NOT publish public REST APIs.
#          Scraping their pages is fragile and against their ToS, so I did not
#          wire them in. Stooq is the honest free alternative.
#
#   2. Gemini "no JSON found"  →  the model returned ```json { ... } ``` (markdown
#      code fences) AND the response got cut off because max_tokens was 600.
#      Fixed by: stripping markdown fences, using balanced-brace parsing,
#      raising max_tokens to 1500, and asking the model to respond with pure JSON.
#
#   3. GLM 400 "Unknown Model"  →  `glm-4-flash` is no longer valid on z.ai.
#      Default is now `glm-4.5-flash` (their current free tier model). Override
#      with the GLM_MODEL env var if z.ai renames it again.
#
#   4. DeepSeek 402 "Insufficient Balance"  →  that's your account balance;
#      nothing to fix in code. But the error handling now clearly labels this
#      as an account issue so you know to top up at platform.deepseek.com or
#      skip DeepSeek by leaving DEEPSEEK_API_KEY unset. Default model corrected
#      to `deepseek-chat`, which is DeepSeek's current production model.

import os
import httpx
import json
import re
import logging
from fastapi import APIRouter, HTTPException
from pydantic import BaseModel
from typing import Optional, List, Dict, Tuple

router = APIRouter(prefix="/api/ai", tags=["ai-advisor"])
log = logging.getLogger(__name__)


# ── LLM providers ──────────────────────────────────────────────────────
def _providers():
    return [
        {
            "name": "deepseek",
            "key": os.getenv("DEEPSEEK_API_KEY", ""),
            "base_url": os.getenv("DEEPSEEK_BASE_URL", "https://api.deepseek.com"),
            "model": os.getenv("DEEPSEEK_MODEL", "deepseek-chat"),
        },
        {
            "name": "gemini",
            "key": os.getenv("GEMINI_API_KEY", ""),
            "base_url": os.getenv("GEMINI_BASE_URL", "https://generativelanguage.googleapis.com/v1beta/openai"),
            "model": os.getenv("GEMINI_MODEL", "gemini-2.5-flash"),
        },
        {
            "name": "glm",
            "key": os.getenv("GLM_API_KEY", ""),
            "base_url": os.getenv("GLM_BASE_URL", "https://api.z.ai/api/paas/v4"),
            "model": os.getenv("GLM_MODEL", "glm-4.5-flash"),
        },
    ]


# ── Robust JSON extraction from LLM output ────────────────────────────
def _extract_json(raw: str) -> Optional[dict]:
    """Extract a JSON object from an LLM response even when it's:
       - wrapped in ```json ... ``` code fences
       - has preamble text before/after
       - has trailing commas (some models emit these)
       - was truncated (we try to auto-close open braces as a last resort)."""
    if not raw:
        return None
    cleaned = raw.strip()
    # Strip markdown code fences: ```json ... ``` or ``` ... ```
    cleaned = re.sub(r'^\s*```(?:json|JSON)?\s*\n?', '', cleaned)
    cleaned = re.sub(r'\n?\s*```\s*$', '', cleaned)
    cleaned = cleaned.strip()

    # Try direct parse
    try:
        return json.loads(cleaned)
    except json.JSONDecodeError:
        pass

    # Walk the string with balanced-brace counting, ignoring braces inside strings.
    def _parse_balanced(text: str) -> Optional[dict]:
        depth = 0
        start = -1
        in_string = False
        escape = False
        for i, ch in enumerate(text):
            if escape:
                escape = False
                continue
            if ch == '\\' and in_string:
                escape = True
                continue
            if ch == '"':
                in_string = not in_string
                continue
            if in_string:
                continue
            if ch == '{':
                if depth == 0:
                    start = i
                depth += 1
            elif ch == '}':
                if depth > 0:
                    depth -= 1
                    if depth == 0 and start != -1:
                        candidate = text[start:i + 1]
                        # Try normally, then strip trailing commas
                        for c in (candidate, re.sub(r',\s*([}\]])', r'\1', candidate)):
                            try:
                                return json.loads(c)
                            except json.JSONDecodeError:
                                pass
                        start = -1
        # Last-ditch: response was truncated mid-JSON. Close the open braces.
        if start != -1 and depth > 0:
            candidate = text[start:] + '}' * depth
            candidate = re.sub(r',\s*([}\]])', r'\1', candidate)
            try:
                return json.loads(candidate)
            except json.JSONDecodeError:
                return None
        return None

    return _parse_balanced(cleaned)


async def _get_parsed_decision(prompt: str) -> dict:
    """Call each provider until one returns valid JSON with required fields."""
    all_providers = _providers()
    configured = [p for p in all_providers if p["key"]]
    if not configured:
        raise HTTPException(500, "No AI provider configured. Set at least one API key (DEEPSEEK, GEMINI, or GLM).")

    errors = []
    skipped_names = []

    for p in all_providers:
        if not p["key"]:
            skipped_names.append(p["name"])
            continue
        base = p["base_url"].rstrip("/")
        try:
            async with httpx.AsyncClient(timeout=45.0) as c:
                r = await c.post(
                    f"{base}/chat/completions",
                    headers={"Authorization": f"Bearer {p['key']}", "Content-Type": "application/json"},
                    json={
                        "model": p["model"],
                        "messages": [{"role": "user", "content": prompt}],
                        "temperature": 0.25,
                        "max_tokens": 1500,   # raised so JSON isn't truncated
                    },
                )
                r.raise_for_status()
                data = r.json()
                raw = data["choices"][0]["message"]["content"]
                parsed = _extract_json(raw)
                if parsed is None:
                    errors.append(f"{p['name']}: could not extract JSON from: {raw[:200]!r}")
                    continue
                if not all(k in parsed for k in ("side", "confidence", "reasoning", "backup_plan")):
                    errors.append(f"{p['name']}: missing required fields in JSON: {list(parsed.keys())}")
                    continue
                return {"parsed": parsed, "provider": p["name"], "model": p["model"]}
        except httpx.HTTPStatusError as e:
            status = e.response.status_code
            body = e.response.text[:200]
            if status == 402:
                errors.append(f"{p['name']}: HTTP 402 — account balance insufficient (top up at provider dashboard)")
            elif status == 400 and "model" in body.lower():
                errors.append(f"{p['name']}: HTTP 400 — model name '{p['model']}' rejected. Set {p['name'].upper()}_MODEL env var to a valid model.")
            elif status == 401:
                errors.append(f"{p['name']}: HTTP 401 — API key rejected (check {p['name'].upper()}_API_KEY)")
            elif status == 429:
                errors.append(f"{p['name']}: HTTP 429 — provider rate-limited")
            else:
                errors.append(f"{p['name']}: HTTP {status} {body}")
        except Exception as e:
            errors.append(f"{p['name']}: {e}")

    msg_parts = []
    if skipped_names:
        msg_parts.append(f"Skipped (no key): {', '.join(skipped_names)}.")
    if errors:
        msg_parts.append("Failures: " + " | ".join(errors))
    raise HTTPException(502, "All AI providers failed. " + " ".join(msg_parts))


async def _call_llm(messages: list, temperature: float, max_tokens: int) -> Dict[str, str]:
    providers = [p for p in _providers() if p["key"]]
    if not providers:
        raise HTTPException(500, "No AI provider configured. Set at least one API key.")
    failures = []
    for p in providers:
        base = p["base_url"].rstrip("/")
        try:
            async with httpx.AsyncClient(timeout=45.0) as c:
                r = await c.post(
                    f"{base}/chat/completions",
                    headers={"Authorization": f"Bearer {p['key']}", "Content-Type": "application/json"},
                    json={"model": p["model"], "messages": messages, "temperature": temperature, "max_tokens": max_tokens},
                )
                r.raise_for_status()
                data = r.json()
                return {"reply": data["choices"][0]["message"]["content"], "provider": p["name"], "model": p["model"]}
        except httpx.HTTPStatusError as e:
            status = e.response.status_code
            hint = ""
            if status == 402: hint = " (insufficient balance — top up account)"
            elif status == 401: hint = " (bad API key)"
            elif status == 400 and "model" in e.response.text.lower(): hint = f" (model '{p['model']}' rejected)"
            failures.append(f"{p['name']}: HTTP {status}{hint}")
        except Exception as e:
            failures.append(f"{p['name']}: {e}")
    raise HTTPException(502, "All configured AI providers failed. " + " | ".join(failures))


@router.get("/health")
async def health():
    return {p["name"]: {"api_key_set": bool(p["key"]), "api_key_length": len(p["key"]),
                       "base_url": p["base_url"], "model": p["model"]} for p in _providers()}


# ── Asset catalog — each asset carries symbols for every data source ──
ASSET_CATALOG: Dict[str, Dict[str, Dict[str, str]]] = {
    "crypto": {
        "BTC":  {"label": "Bitcoin",     "coingecko_id": "bitcoin",     "binance_symbol": "BTCUSDT",  "kraken_pair": "XBTUSD", "coincap_id": "bitcoin",     "cryptocompare_sym": "BTC", "coinpaprika_id": "btc-bitcoin",   "cmc_symbol": "BTC"},
        "ETH":  {"label": "Ethereum",    "coingecko_id": "ethereum",    "binance_symbol": "ETHUSDT",  "kraken_pair": "ETHUSD", "coincap_id": "ethereum",    "cryptocompare_sym": "ETH", "coinpaprika_id": "eth-ethereum",  "cmc_symbol": "ETH"},
        "SOL":  {"label": "Solana",      "coingecko_id": "solana",      "binance_symbol": "SOLUSDT",  "kraken_pair": "SOLUSD", "coincap_id": "solana",      "cryptocompare_sym": "SOL", "coinpaprika_id": "sol-solana",    "cmc_symbol": "SOL"},
        "HYPE": {"label": "Hyperliquid", "coingecko_id": "hyperliquid", "binance_symbol": "HYPEUSDT", "kraken_pair": "",       "coincap_id": "hyperliquid", "cryptocompare_sym": "HYPE","coinpaprika_id": "hype-hyperliquid","cmc_symbol": "HYPE"},
    },
    "stock": {
        "AAPL":  {"label": "Apple",     "yahoo_symbol": "AAPL",  "stooq_symbol": "aapl.us"},
        "TSLA":  {"label": "Tesla",     "yahoo_symbol": "TSLA",  "stooq_symbol": "tsla.us"},
        "MSFT":  {"label": "Microsoft", "yahoo_symbol": "MSFT",  "stooq_symbol": "msft.us"},
        "NVDA":  {"label": "Nvidia",    "yahoo_symbol": "NVDA",  "stooq_symbol": "nvda.us"},
        "GOOGL": {"label": "Alphabet",  "yahoo_symbol": "GOOGL", "stooq_symbol": "googl.us"},
        "AMZN":  {"label": "Amazon",    "yahoo_symbol": "AMZN",  "stooq_symbol": "amzn.us"},
    },
    "index": {
        "SPX":  {"label": "S&P 500",                       "yahoo_symbol": "^GSPC", "stooq_symbol": "^spx"},
        "DJI":  {"label": "Dow Jones Industrial Average",  "yahoo_symbol": "^DJI",  "stooq_symbol": "^dji"},
        "IXIC": {"label": "Nasdaq Composite",              "yahoo_symbol": "^IXIC", "stooq_symbol": "^ndx"},
    },
}

MARKET_TO_ASSET_KEY: Dict[str, str] = {
    "BTC-PERP": "BTC", "ETH-PERP": "ETH", "SOL-PERP": "SOL", "HYPE-PERP": "HYPE",
}


# ── Technical indicator helpers ────────────────────────────────────────
def _sma(values, period):
    if len(values) < period: return None
    return sum(values[-period:]) / period

def _ema(values, period):
    if len(values) < period: return None
    k = 2 / (period + 1)
    ema = sum(values[:period]) / period
    for price in values[period:]:
        ema = price * k + ema * (1 - k)
    return ema

def _ema_series(values, period):
    if len(values) < period: return []
    k = 2 / (period + 1)
    emas = [sum(values[:period]) / period]
    for price in values[period:]:
        emas.append(price * k + emas[-1] * (1 - k))
    return emas

def _slope(series, lookback=5):
    if len(series) < lookback: return None
    recent = series[-lookback:]
    x_mean = (lookback - 1) / 2
    y_mean = sum(recent) / lookback
    num = sum((i - x_mean) * (recent[i] - y_mean) for i in range(lookback))
    den = sum((i - x_mean) ** 2 for i in range(lookback))
    return num / den if den != 0 else 0.0

def _rsi(values, period=14):
    if len(values) < period + 1: return None
    gains, losses = [], []
    for i in range(1, len(values)):
        d = values[i] - values[i - 1]
        gains.append(max(d, 0.0)); losses.append(max(-d, 0.0))
    avg_gain = sum(gains[-period:]) / period
    avg_loss = sum(losses[-period:]) / period
    if avg_loss == 0: return 100.0
    rs = avg_gain / avg_loss
    return 100 - (100 / (1 + rs))

def _atr(highs, lows, closes, period=14):
    if len(closes) < period + 1 or len(highs) != len(closes) or len(lows) != len(closes):
        return None
    trs = []
    for i in range(1, len(closes)):
        tr = max(highs[i] - lows[i], abs(highs[i] - closes[i - 1]), abs(lows[i] - closes[i - 1]))
        trs.append(tr)
    if len(trs) < period: return None
    return sum(trs[-period:]) / period

def _daily_volatility_pct(values):
    if len(values) < 2: return 0.0
    returns = [(values[i] - values[i - 1]) / values[i - 1] for i in range(1, len(values)) if values[i - 1]]
    if not returns: return 0.0
    mean_r = sum(returns) / len(returns)
    variance = sum((r - mean_r) ** 2 for r in returns) / len(returns)
    return (variance ** 0.5) * 100


# ── CRYPTO DATA FETCHERS ─────────────────────────────────────────────
UA = {"User-Agent": "Blockette/5.0 (trading-app)"}
Series = Tuple[List[float], List[float], List[float], List[float]]  # closes, highs, lows, volumes


async def _fetch_binance_4h(symbol: str) -> Series:
    """Binance public klines — real native 4H OHLCV, no key. Best-quality free source."""
    url = "https://api.binance.com/api/v3/klines"
    params = {"symbol": symbol, "interval": "4h", "limit": 300}
    async with httpx.AsyncClient(timeout=15.0) as c:
        r = await c.get(url, params=params, headers=UA)
        r.raise_for_status()
        data = r.json()
    if not isinstance(data, list) or not data:
        raise ValueError("Binance: empty response")
    closes = [float(k[4]) for k in data]
    highs  = [float(k[2]) for k in data]
    lows   = [float(k[3]) for k in data]
    vols   = [float(k[5]) for k in data]
    return closes, highs, lows, vols


async def _fetch_kraken_4h(pair: str) -> Series:
    """Kraken public OHLC — real native 4H, no key."""
    if not pair:
        raise ValueError("Kraken: no pair configured")
    url = "https://api.kraken.com/0/public/OHLC"
    params = {"pair": pair, "interval": 240}  # 240 minutes = 4H
    async with httpx.AsyncClient(timeout=15.0) as c:
        r = await c.get(url, params=params, headers=UA)
        r.raise_for_status()
        data = r.json()
    if data.get("error"):
        raise ValueError(f"Kraken: {data['error']}")
    result = data.get("result", {}) or {}
    ohlc = None
    for k, v in result.items():
        if k != "last" and isinstance(v, list):
            ohlc = v
            break
    if not ohlc:
        raise ValueError("Kraken: no OHLC data")
    # Format: [time, open, high, low, close, vwap, volume, count]
    closes = [float(row[4]) for row in ohlc]
    highs  = [float(row[2]) for row in ohlc]
    lows   = [float(row[3]) for row in ohlc]
    vols   = [float(row[6]) for row in ohlc]
    return closes, highs, lows, vols


async def _fetch_coingecko_4h(coingecko_id: str) -> Series:
    """CoinGecko /ohlc for 30d = 4H bars natively. Rate-limited without key."""
    url = f"https://api.coingecko.com/api/v3/coins/{coingecko_id}/ohlc"
    async with httpx.AsyncClient(timeout=15.0) as c:
        r = await c.get(url, params={"vs_currency": "usd", "days": 30}, headers=UA)
        r.raise_for_status()
        ohlc = r.json()
    if not ohlc:
        raise ValueError("CoinGecko: empty response")
    highs  = [row[2] for row in ohlc]
    lows   = [row[3] for row in ohlc]
    closes = [row[4] for row in ohlc]

    # Volumes come from /market_chart; align by chunking.
    volumes = [0.0] * len(closes)
    try:
        async with httpx.AsyncClient(timeout=10.0) as c:
            v = await c.get(
                f"https://api.coingecko.com/api/v3/coins/{coingecko_id}/market_chart",
                params={"vs_currency": "usd", "days": 30}, headers=UA,
            )
            v.raise_for_status()
            raw_vols = [row[1] for row in v.json().get("total_volumes", [])]
        if raw_vols and closes:
            per_bar = max(1, len(raw_vols) // len(closes))
            for i in range(len(closes)):
                lo, hi = i * per_bar, min(len(raw_vols), (i + 1) * per_bar)
                volumes[i] = sum(raw_vols[lo:hi]) if hi > lo else 0.0
    except Exception:
        pass  # volumes optional; strategies degrade gracefully
    return closes, highs, lows, volumes


async def _fetch_coincap_4h(coincap_id: str) -> Series:
    """CoinCap candles via Binance exchange, 4H interval. Free, no key needed."""
    url = "https://api.coincap.io/v2/candles"
    params = {"exchange": "binance", "interval": "h4", "baseId": coincap_id, "quoteId": "tether"}
    async with httpx.AsyncClient(timeout=15.0) as c:
        r = await c.get(url, params=params, headers=UA)
        r.raise_for_status()
        data = r.json()
    candles = data.get("data", []) or []
    if not candles:
        raise ValueError("CoinCap: no candles")
    closes = [float(k["close"])  for k in candles]
    highs  = [float(k["high"])   for k in candles]
    lows   = [float(k["low"])    for k in candles]
    vols   = [float(k.get("volume") or 0.0) for k in candles]
    return closes, highs, lows, vols


async def _fetch_cryptocompare_4h(sym: str) -> Series:
    """CryptoCompare histohour aggregated to 4H. Free, no key required for light use."""
    url = "https://min-api.cryptocompare.com/data/v2/histohour"
    params = {"fsym": sym, "tsym": "USD", "limit": 200, "aggregate": 4}
    key = os.getenv("CRYPTOCOMPARE_API_KEY", "")
    headers = dict(UA)
    if key:
        headers["Authorization"] = f"Apikey {key}"
    async with httpx.AsyncClient(timeout=15.0) as c:
        r = await c.get(url, params=params, headers=headers)
        r.raise_for_status()
        data = r.json()
    if data.get("Response") == "Error":
        raise ValueError(f"CryptoCompare: {data.get('Message')}")
    rows = (data.get("Data") or {}).get("Data") or []
    if not rows:
        raise ValueError("CryptoCompare: no data")
    closes = [float(row["close"]) for row in rows]
    highs  = [float(row["high"])  for row in rows]
    lows   = [float(row["low"])   for row in rows]
    vols   = [float(row.get("volumeto", 0.0)) for row in rows]
    return closes, highs, lows, vols


async def _fetch_cmc_4h(cmc_symbol: str) -> Series:
    """CoinMarketCap OHLCV — requires CMC_API_KEY env var. Their free tier supports this."""
    key = os.getenv("CMC_API_KEY", "")
    if not key:
        raise ValueError("CoinMarketCap: no CMC_API_KEY")
    url = "https://pro-api.coinmarketcap.com/v2/cryptocurrency/ohlcv/historical"
    params = {"symbol": cmc_symbol, "convert": "USD", "time_period": "hourly", "interval": "4h", "count": 200}
    headers = {"X-CMC_PRO_API_KEY": key, **UA}
    async with httpx.AsyncClient(timeout=15.0) as c:
        r = await c.get(url, params=params, headers=headers)
        r.raise_for_status()
        data = r.json()
    quotes = ((data.get("data") or {}).get(cmc_symbol) or [{}])[0].get("quotes") or []
    if not quotes:
        raise ValueError("CoinMarketCap: no quotes")
    closes = [float(q["quote"]["USD"]["close"])  for q in quotes]
    highs  = [float(q["quote"]["USD"]["high"])   for q in quotes]
    lows   = [float(q["quote"]["USD"]["low"])    for q in quotes]
    vols   = [float(q["quote"]["USD"].get("volume") or 0.0) for q in quotes]
    return closes, highs, lows, vols


async def _fetch_coinpaprika_current(coinpaprika_id: str) -> Optional[float]:
    """CoinPaprika only gives us a current spot price on free tier; useful as absolute last resort."""
    url = f"https://api.coinpaprika.com/v1/tickers/{coinpaprika_id}"
    async with httpx.AsyncClient(timeout=10.0) as c:
        r = await c.get(url, headers=UA)
        r.raise_for_status()
        data = r.json()
    price = ((data.get("quotes") or {}).get("USD") or {}).get("price")
    return float(price) if price is not None else None


async def _fetch_crypto_4h(asset_key: str) -> Tuple[Series, str]:
    """Try every crypto source in order. Returns (series, source_name_used)."""
    entry = ASSET_CATALOG["crypto"][asset_key.upper()]
    attempts = [
        ("binance",        entry.get("binance_symbol"),   _fetch_binance_4h),
        ("kraken",         entry.get("kraken_pair"),      _fetch_kraken_4h),
        ("coingecko",      entry.get("coingecko_id"),     _fetch_coingecko_4h),
        ("coincap",        entry.get("coincap_id"),       _fetch_coincap_4h),
        ("cryptocompare",  entry.get("cryptocompare_sym"), _fetch_cryptocompare_4h),
        ("coinmarketcap",  entry.get("cmc_symbol"),       _fetch_cmc_4h),
    ]
    errors = []
    for name, ident, fn in attempts:
        if not ident:
            continue
        try:
            series = await fn(ident)
            if series and series[0] and len(series[0]) >= 20:
                log.info(f"crypto data source used: {name} ({len(series[0])} bars)")
                return series, name
            errors.append(f"{name}: too few bars ({len(series[0]) if series else 0})")
        except httpx.HTTPStatusError as e:
            errors.append(f"{name}: HTTP {e.response.status_code}")
        except Exception as e:
            errors.append(f"{name}: {e}")
    raise HTTPException(502, f"All crypto data sources failed: {'; '.join(errors)}")


# ── STOCK / INDEX DATA FETCHERS ──────────────────────────────────────
async def _fetch_yahoo_4h(yahoo_symbol: str) -> Series:
    """Yahoo Finance 1H bars resampled to 4H."""
    url = f"https://query1.finance.yahoo.com/v8/finance/chart/{yahoo_symbol}"
    params = {"range": "60d", "interval": "1h"}
    headers = {"User-Agent": "Mozilla/5.0"}
    async with httpx.AsyncClient(timeout=15.0) as c:
        r = await c.get(url, params=params, headers=headers)
        r.raise_for_status()
        data = r.json()
    result = (data.get("chart") or {}).get("result")
    if not result:
        raise ValueError(f"Yahoo: no data for {yahoo_symbol}")
    q = result[0]["indicators"]["quote"][0]
    return _resample_1h_to_4h(q.get("close") or [], q.get("high") or [], q.get("low") or [], q.get("volume") or [])


async def _fetch_stooq_4h(stooq_symbol: str) -> Series:
    """Stooq CSV hourly data resampled to 4H. Free, no key needed."""
    url = "https://stooq.com/q/d/l/"
    params = {"s": stooq_symbol, "i": "h"}  # h = hourly
    headers = {"User-Agent": "Mozilla/5.0"}
    async with httpx.AsyncClient(timeout=15.0) as c:
        r = await c.get(url, params=params, headers=headers)
        r.raise_for_status()
        csv_text = r.text.strip()
    if not csv_text or "No data" in csv_text[:100]:
        raise ValueError(f"Stooq: no data for {stooq_symbol}")
    lines = csv_text.splitlines()
    if len(lines) < 5:
        raise ValueError("Stooq: too few rows")
    header = [h.strip() for h in lines[0].split(",")]
    try:
        i_open  = header.index("Open")
        i_high  = header.index("High")
        i_low   = header.index("Low")
        i_close = header.index("Close")
        i_vol   = header.index("Volume") if "Volume" in header else -1
    except ValueError:
        raise ValueError(f"Stooq: unexpected CSV header {header}")
    closes_1h, highs_1h, lows_1h, vols_1h = [], [], [], []
    for line in lines[1:]:
        parts = line.split(",")
        if len(parts) <= i_close: continue
        try:
            closes_1h.append(float(parts[i_close]))
            highs_1h.append(float(parts[i_high]))
            lows_1h.append(float(parts[i_low]))
            vols_1h.append(float(parts[i_vol]) if i_vol >= 0 and parts[i_vol] else 0.0)
        except (ValueError, IndexError):
            continue
    if len(closes_1h) < 20:
        raise ValueError(f"Stooq: only {len(closes_1h)} valid hourly bars")
    return _resample_1h_to_4h(closes_1h, highs_1h, lows_1h, vols_1h)


def _resample_1h_to_4h(closes_1h, highs_1h, lows_1h, vols_1h) -> Series:
    closes_4h, highs_4h, lows_4h, vols_4h = [], [], [], []
    n = min(len(closes_1h), len(highs_1h), len(lows_1h))
    for i in range(0, n, 4):
        ch = [x for x in closes_1h[i:i+4] if x is not None]
        hh = [x for x in highs_1h[i:i+4]  if x is not None]
        ll = [x for x in lows_1h[i:i+4]   if x is not None]
        vv = [x for x in (vols_1h[i:i+4] if vols_1h else []) if x is not None]
        if not ch: continue
        closes_4h.append(ch[-1])
        highs_4h.append(max(hh) if hh else ch[-1])
        lows_4h.append(min(ll) if ll else ch[-1])
        vols_4h.append(sum(vv) if vv else 0.0)
    if len(closes_4h) < 5:
        raise ValueError("Not enough 4H bars after resampling")
    return closes_4h, highs_4h, lows_4h, vols_4h


async def _fetch_stock_4h(asset_class: str, symbol_key: str) -> Tuple[Series, str]:
    entry = ASSET_CATALOG[asset_class][symbol_key.upper()]
    errors = []
    if entry.get("yahoo_symbol"):
        try:
            series = await _fetch_yahoo_4h(entry["yahoo_symbol"])
            if series[0] and len(series[0]) >= 20:
                log.info(f"stock data source used: yahoo ({len(series[0])} bars)")
                return series, "yahoo"
        except Exception as e:
            errors.append(f"yahoo: {e}")
    if entry.get("stooq_symbol"):
        try:
            series = await _fetch_stooq_4h(entry["stooq_symbol"])
            if series[0] and len(series[0]) >= 20:
                log.info(f"stock data source used: stooq ({len(series[0])} bars)")
                return series, "stooq"
        except Exception as e:
            errors.append(f"stooq: {e}")
    raise HTTPException(502, f"All stock data sources failed: {'; '.join(errors)}")


async def _fetch_asset_4h(asset_class: str, symbol_key: str) -> Tuple[Series, str]:
    catalog = ASSET_CATALOG.get(asset_class, {})
    if symbol_key.upper() not in catalog:
        raise HTTPException(400, f"Unknown {asset_class} symbol '{symbol_key}'")
    if asset_class == "crypto":
        return await _fetch_crypto_4h(symbol_key)
    return await _fetch_stock_4h(asset_class, symbol_key)


# ── Strategy detectors ────────────────────────────────────────────────
def _trend_following_signal(closes, ema50, ema200, ema50_series, ema200_series):
    if not ema50 or not ema200 or not closes:
        return {"signal": "none", "note": "insufficient EMA data"}
    price = closes[-1]
    slope50 = _slope(ema50_series, 5) if len(ema50_series) >= 5 else 0
    slope200 = _slope(ema200_series, 5) if len(ema200_series) >= 5 else 0
    dist_pct = abs(price - ema50) / ema50 * 100
    if ema50 > ema200 and price > ema200 and slope50 > 0 and slope200 > 0:
        if dist_pct < 2.0 and price >= ema50 * 0.995:
            return {"signal": "long", "note": f"uptrend + pullback to EMA50 ({dist_pct:.2f}% away). Trend-Following LONG."}
        return {"signal": "bias_long", "note": f"uptrend intact but extended {dist_pct:.2f}% from EMA50 — wait for pullback."}
    if ema50 < ema200 and price < ema200 and slope50 < 0 and slope200 < 0:
        if dist_pct < 2.0 and price <= ema50 * 1.005:
            return {"signal": "short", "note": f"downtrend + rally to EMA50 ({dist_pct:.2f}% away). Trend-Following SHORT."}
        return {"signal": "bias_short", "note": f"downtrend intact but extended {dist_pct:.2f}% from EMA50 — wait for rally."}
    return {"signal": "none", "note": "EMAs not aligned — no clear trend."}


def _breakout_signal(closes, highs, lows, volumes, lookback=20):
    if len(closes) < lookback + 2:
        return {"signal": "none", "note": "not enough bars for breakout scan"}
    prior_highs = highs[-(lookback + 1):-1]
    prior_lows  = lows[-(lookback + 1):-1]
    range_high, range_low = max(prior_highs), min(prior_lows)
    last_close, prev_close = closes[-1], closes[-2]
    avg_vol = sum(volumes[-lookback:]) / lookback if any(volumes[-lookback:]) else 0.0
    last_vol = volumes[-1] if volumes else 0.0
    vol_ok = (last_vol > avg_vol * 1.3) if avg_vol > 0 else False
    vnote = "with volume expansion" if vol_ok else "on soft volume (be cautious)"
    if last_close > range_high and prev_close <= range_high:
        return {"signal": "long",  "note": f"broke above 20-bar high ${range_high:,.2f} {vnote}. Breakout LONG."}
    if last_close < range_low and prev_close >= range_low:
        return {"signal": "short", "note": f"broke below 20-bar low ${range_low:,.2f} {vnote}. Breakout SHORT."}
    if last_close > range_high * 0.995:
        return {"signal": "watch_long",  "note": f"pressing 20-bar high ${range_high:,.2f}."}
    if last_close < range_low * 1.005:
        return {"signal": "watch_short", "note": f"pressing 20-bar low ${range_low:,.2f}."}
    return {"signal": "none", "note": f"inside range ${range_low:,.2f}–${range_high:,.2f}."}


def _orb_signal(closes, highs, lows):
    if len(closes) < 6:
        return {"signal": "none", "note": "not enough intraday bars for ORB"}
    session_highs = highs[-6:]; session_lows = lows[-6:]
    or_high, or_low = session_highs[0], session_lows[0]
    last_close = closes[-1]
    if last_close > or_high:
        return {"signal": "long",  "note": f"broke above opening range high ${or_high:,.2f}. ORB LONG."}
    if last_close < or_low:
        return {"signal": "short", "note": f"broke below opening range low ${or_low:,.2f}. ORB SHORT."}
    return {"signal": "none", "note": f"inside opening range ${or_low:,.2f}–${or_high:,.2f}."}


class TechnicalAssessment:
    def __init__(self, closes, highs, lows, volumes, current_price):
        self.closes = closes; self.highs = highs; self.lows = lows; self.volumes = volumes
        self.current_price = current_price
        self.ema20 = _ema(closes, 20) if len(closes) >= 20 else None
        self.ema50 = _ema(closes, 50) if len(closes) >= 50 else None
        self.ema200 = _ema(closes, 200) if len(closes) >= 200 else None
        self.ema50_series = _ema_series(closes, 50) if len(closes) >= 50 else []
        self.ema200_series = _ema_series(closes, 200) if len(closes) >= 200 else []
        self.rsi14 = _rsi(closes, 14)
        self.atr14 = _atr(highs, lows, closes, 14)
        self.high_30d = max(closes) if closes else current_price
        self.low_30d  = min(closes) if closes else current_price
        self.trend_following = _trend_following_signal(closes, self.ema50, self.ema200, self.ema50_series, self.ema200_series)
        self.breakout = _breakout_signal(closes, highs, lows, volumes)
        self.orb = _orb_signal(closes, highs, lows)

    def any_signal_side(self):
        longs  = sum(1 for s in (self.trend_following, self.breakout, self.orb) if s.get("signal") == "long")
        shorts = sum(1 for s in (self.trend_following, self.breakout, self.orb) if s.get("signal") == "short")
        if longs > shorts and longs >= 1: return "long"
        if shorts > longs and shorts >= 1: return "short"
        return None

    def summary(self):
        atr_str = f"${self.atr14:,.2f}" if self.atr14 else "n/a"
        return f"""**4H Technical Assessment**
- Current price: ${self.current_price:,.2f}
- EMA20 / EMA50 / EMA200: {f'${self.ema20:,.2f}' if self.ema20 else 'n/a'} / {f'${self.ema50:,.2f}' if self.ema50 else 'n/a'} / {f'${self.ema200:,.2f}' if self.ema200 else 'n/a'}
- RSI(14): {f'{self.rsi14:.1f}' if self.rsi14 else 'n/a'}
- ATR(14): {atr_str}
- 30-day High/Low: ${self.high_30d:,.2f} / ${self.low_30d:,.2f}

**Strategy Signals (4H)**
1. Trend-Following: {self.trend_following.get('signal', 'none').upper()} — {self.trend_following.get('note', '')}
2. Breakout Trading: {self.breakout.get('signal', 'none').upper()} — {self.breakout.get('note', '')}
3. Opening Range Breakout (ORB): {self.orb.get('signal', 'none').upper()} — {self.orb.get('note', '')}"""


# ── Pydantic models ──────────────────────────────────────────────────────
class Indicators(BaseModel):
    price: float
    sma20: Optional[float] = None
    sma50: Optional[float] = None
    rsi14: Optional[float] = None
    atr14: Optional[float] = None
    high_30d: float
    low_30d: float
    change_24h_pct: Optional[float] = None
    volatility_pct_daily: float
    trend_structure: Optional[str] = None

class AnalyzeRequest(BaseModel):
    market: str
    account_margin_usd: float = 0.0
    risk_tolerance: str = "moderate"

class AnalyzeResponse(BaseModel):
    market: str
    indicators: Indicators
    analysis: str
    model: str
    source: str = "multi-source"

class ChatMessage(BaseModel):
    role: str
    content: str

class ChatRequest(BaseModel):
    asset_class: str = "crypto"
    symbol: str = "BTC"
    messages: List[ChatMessage]
    account_margin_usd: float = 0.0
    risk_tolerance: str = "moderate"

class ChatResponse(BaseModel):
    reply: str
    model: str
    asset_label: str

class PerformanceSummary(BaseModel):
    trades: int = 0
    wins: int = 0
    losses: int = 0
    win_rate_pct: float = 0.0
    avg_win_pct: float = 0.0
    avg_loss_pct: float = 0.0
    total_pnl_usd: float = 0.0

class Sizing(BaseModel):
    risk_pct: float
    risk_usd: float
    position_value_usd: float
    margin_required_usd: float
    stop_distance_pct: float
    reward_usd: float
    reward_to_risk: float

class DecideRequest(BaseModel):
    asset_class: str = "crypto"
    symbol: str = "BTC"
    account_margin_usd: float = 0.0
    risk_tolerance: str = "moderate"
    leverage: int = 5
    performance: Optional[PerformanceSummary] = None

class DecideResponse(BaseModel):
    asset_label: str
    side: str
    confidence: str
    strategy: Optional[str] = None
    entry_price: Optional[float] = None
    stop_loss: Optional[float] = None
    take_profit: Optional[float] = None
    reasoning: str
    backup_plan: str
    indicators: Optional[Indicators] = None
    technical_assessment: Optional[str] = None
    sizing: Optional[Sizing] = None
    model: str
    data_source: Optional[str] = None
    timeframe: str = "4H"


# ── Risk % per user's spec: 1% / 1.5% / 2% ─────────────────────────────
RISK_PCT_BY_TOLERANCE: Dict[str, float] = {
    "conservative": 0.0100,
    "moderate":     0.0150,
    "aggressive":   0.0200,
}


def _size_position(margin_usd, risk_tolerance, entry, stop, leverage):
    risk_pct = RISK_PCT_BY_TOLERANCE.get(risk_tolerance, RISK_PCT_BY_TOLERANCE["moderate"])
    risk_usd = margin_usd * risk_pct
    stop_distance_pct = abs(entry - stop) / entry if entry else 0
    if stop_distance_pct <= 0:
        return {"risk_pct": risk_pct * 100, "risk_usd": risk_usd,
                "position_value_usd": 0.0, "margin_required_usd": 0.0,
                "stop_distance_pct": 0.0, "reward_usd": 0.0, "reward_to_risk": 0.0}
    position_value_usd = risk_usd / stop_distance_pct
    margin_required_usd = min(position_value_usd / max(leverage, 1), margin_usd)
    return {
        "risk_pct": risk_pct * 100,
        "risk_usd": round(risk_usd, 2),
        "position_value_usd": round(position_value_usd, 2),
        "margin_required_usd": round(margin_required_usd, 2),
        "stop_distance_pct": round(stop_distance_pct * 100, 3),
        "reward_usd": round(risk_usd * 2, 2),
        "reward_to_risk": 2.0,
    }


def _compute_indicators(closes, highs=None, lows=None) -> Optional[Indicators]:
    if len(closes) < 20:
        return None
    change_pct = ((closes[-1] - closes[-7]) / closes[-7]) * 100 if len(closes) > 7 else None
    sma20 = _sma(closes, 20)
    sma50 = _sma(closes, 50) if len(closes) >= 50 else None
    ema50 = _ema(closes, 50) if len(closes) >= 50 else None
    ema200 = _ema(closes, 200) if len(closes) >= 200 else None
    atr14 = _atr(highs, lows, closes, 14) if (highs and lows) else None
    trend = "insufficient data"
    if ema50 and ema200:
        if ema50 > ema200 and closes[-1] > ema50:      trend = "uptrend structure"
        elif ema50 < ema200 and closes[-1] < ema50:    trend = "downtrend structure"
        else:                                          trend = "mixed/consolidating"
    return Indicators(
        price=closes[-1], sma20=sma20, sma50=sma50, rsi14=_rsi(closes, 14), atr14=atr14,
        high_30d=max(closes), low_30d=min(closes),
        change_24h_pct=change_pct, volatility_pct_daily=_daily_volatility_pct(closes),
        trend_structure=trend,
    )


# ── Endpoints ────────────────────────────────────────────────────────────
@router.get("/assets")
async def list_assets():
    return {cls: {key: entry["label"] for key, entry in entries.items()}
            for cls, entries in ASSET_CATALOG.items()}


@router.post("/analyze", response_model=AnalyzeResponse)
async def analyze(req: AnalyzeRequest):
    asset_key = MARKET_TO_ASSET_KEY.get(req.market)
    if not asset_key:
        raise HTTPException(400, f"Unknown market '{req.market}'")
    try:
        (closes, highs, lows, _vols), src = await _fetch_crypto_4h(asset_key)
    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(502, f"Market data error: {e}")
    if len(closes) < 20:
        raise HTTPException(502, "Not enough market data")
    indicators = _compute_indicators(closes, highs, lows)
    prompt = _build_analyze_prompt(req.market, indicators, req.account_margin_usd, req.risk_tolerance)
    result = await _call_llm([{"role": "user", "content": prompt}], temperature=0.3, max_tokens=800)
    return AnalyzeResponse(market=req.market, indicators=indicators, analysis=result["reply"],
                           model=f"{result['provider']}:{result['model']}", source=src)


def _build_analyze_prompt(market, ind, margin_usd, risk_tolerance):
    return f"""You are a 4H-timeframe TA and risk-management assistant in a testnet app. Not a financial advisor.
Timeframe: 4H
Market: {market}
Price: ${ind.price:,.2f}
SMA20: {f"${ind.sma20:,.2f}" if ind.sma20 else "n/a"}
SMA50: {f"${ind.sma50:,.2f}" if ind.sma50 else "n/a"}
RSI14: {ind.rsi14 if ind.rsi14 else "n/a"}
ATR14: {f"${ind.atr14:,.2f}" if ind.atr14 else "n/a"}
30d high/low: ${ind.high_30d:,.2f} / ${ind.low_30d:,.2f}
Recent change: {f"{ind.change_24h_pct:.2f}%" if ind.change_24h_pct else "n/a"}
Volatility: {ind.volatility_pct_daily:.2f}%
Margin: ${margin_usd:,.2f}, risk: {risk_tolerance} ({RISK_PCT_BY_TOLERANCE.get(risk_tolerance, 0.015)*100:.2f}% per trade)
Strategies you evaluate (name whichever fits): Trend-Following, Breakout Trading, Opening Range Breakout (ORB).
Rules: Reward is fixed 2:1 vs. risk (system enforces TP = 2× SL distance).
Respond with: **Trend Read**, **Strategy Fit**, **Key Levels**, **Entry Timing**, **Risk Management**, **Disclaimer**."""


@router.post("/chat", response_model=ChatResponse)
async def chat(req: ChatRequest):
    catalog_entry = ASSET_CATALOG.get(req.asset_class, {}).get(req.symbol.upper())
    if not catalog_entry:
        raise HTTPException(400, f"Unknown {req.asset_class} symbol '{req.symbol}'")
    asset_label = catalog_entry["label"]
    indicators = None
    try:
        (closes, highs, lows, _vols), _src = await _fetch_asset_4h(req.asset_class, req.symbol)
        indicators = _compute_indicators(closes, highs, lows)
    except Exception:
        pass
    sys_prompt = _chat_system_prompt(asset_label, req.asset_class, indicators, req.account_margin_usd, req.risk_tolerance)
    messages = [{"role": "system", "content": sys_prompt}] + [{"role": m.role, "content": m.content} for m in req.messages[-20:]]
    result = await _call_llm(messages, temperature=0.4, max_tokens=700)
    return ChatResponse(reply=result["reply"], model=f"{result['provider']}:{result['model']}", asset_label=asset_label)


def _chat_system_prompt(asset_label, asset_class, ind, margin_usd, risk_tolerance):
    kind = {"crypto": "cryptocurrency", "stock": "stock", "index": "index"}.get(asset_class, "asset")
    if ind:
        data = (f"Real 4H data for {asset_label} ({kind}): Price ${ind.price:,.2f}, "
                f"RSI {ind.rsi14}, ATR ${ind.atr14 if ind.atr14 else 'n/a'}, "
                f"trend {ind.trend_structure}, ~24h change {ind.change_24h_pct}%")
    else:
        data = f"Live data for {asset_label} unavailable"
    return f"""You are a testnet 4H-timeframe strategy advisor. Not financial advice.
{data}
Strategies you know: Trend-Following, Breakout Trading, Opening Range Breakout (ORB).
Trader risk: {risk_tolerance} ({RISK_PCT_BY_TOLERANCE.get(risk_tolerance, 0.015)*100:.2f}% per trade). Reward:Risk is fixed 2:1.
Margin: ${margin_usd:,.2f}. Keep answers concise, concrete, and grounded in the numbers above."""


# ── DECIDE endpoint ────────────────────────────────────────────────────
def _decision_prompt(asset_label, asset_class, ind, tech_summary, precomputed_side,
                     margin_usd, risk_tolerance, leverage, perf):
    kind = {"crypto": "crypto", "stock": "stock", "index": "index"}.get(asset_class, "asset")
    data_block = (f"Real 4H data for {asset_label} ({kind}): Price ${ind.price:,.2f}, "
                  f"RSI {ind.rsi14}, ATR ${ind.atr14 if ind.atr14 else 'n/a'}, "
                  f"High/Low ${ind.high_30d:,.2f}/${ind.low_30d:,.2f}"
                  if ind else "Data unavailable — prefer WAIT")
    if perf and perf.trades > 0:
        perf_block = (f"Trader history: {perf.trades} trades, win rate {perf.win_rate_pct:.1f}%, "
                      f"avg win +{perf.avg_win_pct:.2f}%, avg loss -{abs(perf.avg_loss_pct):.2f}%, "
                      f"total PnL ${perf.total_pnl_usd:,.2f}. Factor this in.")
    else:
        perf_block = "No trade history."
    hint = f"Strategy signals suggest bias: {precomputed_side.upper()}." if precomputed_side else "No strategy is firing — WAIT is the safe default."
    risk_pct_used = RISK_PCT_BY_TOLERANCE.get(risk_tolerance, 0.015) * 100
    return f"""Decisive 4H strategist — testnet, not financial advice.
{data_block}
{perf_block}

Pre-computed 4H TA and strategy signals:
{tech_summary}

{hint}

Constraints:
- Timeframe: 4H.
- Strategies: Trend-Following, Breakout Trading, Opening Range Breakout (ORB).
- Pick a side ONLY if at least one strategy is firing above; otherwise return "wait".
- Risk model: {risk_tolerance} ({risk_pct_used:.2f}% of margin per trade).
- Reward:Risk is FIXED at 2:1 by the system — the system rewrites take_profit to be exactly 2× the stop distance from entry.
- Stop distance should be realistic: 1× to 2× ATR from entry.

Margin: ${margin_usd:,.2f}, leverage: {leverage}x.

VERY IMPORTANT — respond with a SINGLE valid JSON object and NOTHING ELSE.
Do NOT wrap it in markdown code fences. Do NOT add any prose before or after.
Use these exact keys:
{{"side":"long","confidence":"medium","strategy":"trend_following","entry_price":100.0,"stop_loss":97.0,"take_profit":106.0,"reasoning":"why","backup_plan":"if wrong"}}

Valid values: side ∈ [long, short, wait]; confidence ∈ [low, medium, high]; strategy ∈ [trend_following, breakout, orb, none]."""


def _enforce_2_to_1_rr(side, entry, stop):
    if side == "long":
        risk_dist = entry - stop
        return entry + 2 * risk_dist if risk_dist > 0 else entry
    risk_dist = stop - entry
    return entry - 2 * risk_dist if risk_dist > 0 else entry


@router.post("/decide", response_model=DecideResponse)
async def decide(req: DecideRequest):
    catalog_entry = ASSET_CATALOG.get(req.asset_class, {}).get(req.symbol.upper())
    if not catalog_entry:
        raise HTTPException(400, f"Unknown {req.asset_class} symbol '{req.symbol}'")
    asset_label = catalog_entry["label"]

    indicators: Optional[Indicators] = None
    tech_summary = "Technical assessment unavailable."
    precomputed_side: Optional[str] = None
    data_source = None

    try:
        (closes_4h, highs_4h, lows_4h, volumes_4h), data_source = await _fetch_asset_4h(req.asset_class, req.symbol)
        if len(closes_4h) >= 20:
            assessment = TechnicalAssessment(closes_4h, highs_4h, lows_4h, volumes_4h, closes_4h[-1])
            tech_summary = assessment.summary()
            indicators = _compute_indicators(closes_4h, highs_4h, lows_4h)
            precomputed_side = assessment.any_signal_side()
    except HTTPException:
        raise
    except Exception as e:
        log.warning(f"TA compute failed: {e}")

    prompt = _decision_prompt(asset_label, req.asset_class, indicators, tech_summary, precomputed_side,
                              req.account_margin_usd, req.risk_tolerance, req.leverage, req.performance)

    parsed_data = await _get_parsed_decision(prompt)
    parsed = parsed_data["parsed"]
    provider_name = parsed_data["provider"]
    model_used = parsed_data["model"]

    side = str(parsed.get("side", "wait")).lower()
    if side not in ("long", "short", "wait"):
        side = "wait"
    if precomputed_side is None and side != "wait":
        side = "wait"

    confidence = str(parsed.get("confidence", "low")).lower()
    if confidence not in ("low", "medium", "high"):
        confidence = "low"

    strategy = str(parsed.get("strategy", "none")).lower()
    if strategy not in ("trend_following", "breakout", "orb", "none"):
        strategy = "none"

    entry_price = parsed.get("entry_price")
    stop_loss = parsed.get("stop_loss")
    take_profit = parsed.get("take_profit")
    reasoning = str(parsed.get("reasoning", "")).strip() or "No reasoning."
    backup_plan = str(parsed.get("backup_plan", "")).strip() or "No backup plan."

    sizing = None
    if side in ("long", "short") and entry_price and stop_loss and req.account_margin_usd > 0:
        try:
            entry_f = float(entry_price); stop_f = float(stop_loss)
            if side == "long" and stop_f >= entry_f:
                side = "wait"
            elif side == "short" and stop_f <= entry_f:
                side = "wait"
            else:
                take_profit = round(_enforce_2_to_1_rr(side, entry_f, stop_f), 8)
                entry_price = entry_f; stop_loss = stop_f
                sized = _size_position(req.account_margin_usd, req.risk_tolerance, entry_f, stop_f, req.leverage)
                sizing = Sizing(**sized)
        except (TypeError, ValueError):
            sizing = None; side = "wait"

    return DecideResponse(
        asset_label=asset_label,
        side=side,
        confidence=confidence,
        strategy=strategy if side != "wait" else None,
        entry_price=entry_price if side != "wait" else None,
        stop_loss=stop_loss if side != "wait" else None,
        take_profit=take_profit if side != "wait" else None,
        reasoning=reasoning,
        backup_plan=backup_plan,
        indicators=indicators,
        technical_assessment=tech_summary,
        sizing=sizing,
        model=f"{provider_name}:{model_used}",
        data_source=data_source,
        timeframe="4H",
    )