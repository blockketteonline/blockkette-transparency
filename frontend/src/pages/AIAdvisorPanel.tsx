// ciphervault-frontend/src/pages/AIAdvisorPanel.tsx
// ── AI trade analysis panel — 4H timeframe, three strategies, 2:1 R:R ──────
//
// Same idea as before, but now:
//   • Timeframe fixed to 4H everywhere (backend fetches 4H bars for both crypto
//     and stocks/indices).
//   • Risk %: 1% / 1.5% / 2% (conservative / moderate / aggressive).
//   • The model evaluates three strategies: Trend-Following, Breakout, ORB.
//   • Auto-analyze on market change so the user gets a fresh read the moment
//     they pick a different asset.

import { useEffect, useState } from 'react';

const C = {
  BG: '#050807', SURF: '#0B120E', SURF2: '#101B15',
  TEAL: '#00FF6A', TEALD: '#00CC55', TEALG: 'rgba(0,255,106,0.10)',
  TEXT: '#EAFCEF', TEXT2: '#8FA79B', TEXT3: '#4C5F56',
  GREEN: '#00E5A0', RED: '#FF4D6A', GOLD: '#FFD700', AMBER: '#F59E0B',
  BDR: 'rgba(0,255,106,0.14)', BDR2: 'rgba(255,255,255,0.07)',
} as const;

type RiskTolerance = 'conservative' | 'moderate' | 'aggressive';

interface Indicators {
  price: number;
  sma20: number | null;
  sma50: number | null;
  rsi14: number | null;
  atr14: number | null;
  high_30d: number;
  low_30d: number;
  change_24h_pct: number | null;
  volatility_pct_daily: number;
}

interface AnalyzeResult {
  market: string;
  indicators: Indicators;
  analysis: string;
  model: string;
  source: string;
}

const MARKET_OPTIONS = [
  { id: 'BTC-PERP', sym: 'BTC', icon: '₿', color: '#F7931A' },
  { id: 'ETH-PERP', sym: 'ETH', icon: '⟠', color: '#627EEA' },
  { id: 'SOL-PERP', sym: 'SOL', icon: '◎', color: '#9945FF' },
  { id: 'HYPE-PERP', sym: 'HYPE', icon: '⚡', color: '#00D2FF' },
];

const RISK_LABEL: Record<RiskTolerance, string> = {
  conservative: '1.00%',
  moderate:     '1.50%',
  aggressive:   '2.00%',
};

const fmtUSD = (n: number) => new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD', maximumFractionDigits: n < 10 ? 4 : 2 }).format(n);

interface AIAdvisorPanelProps {
  backendUrl: string;
  defaultMarket?: string;
  marginUsd?: number;
  onApplySuggestion?: (side: 'long' | 'short' | null) => void;
  autoAnalyze?: boolean;
}

const AIAdvisorPanel = ({
  backendUrl, defaultMarket = 'BTC-PERP', marginUsd = 0, onApplySuggestion, autoAnalyze = true,
}: AIAdvisorPanelProps) => {
  const [market, setMarket] = useState(defaultMarket);
  const [riskTolerance, setRiskTolerance] = useState<RiskTolerance>('moderate');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [result, setResult] = useState<AnalyzeResult | null>(null);

  const runAnalysis = async () => {
    setLoading(true); setError(''); setResult(null);
    try {
      const r = await fetch(`${backendUrl}/api/ai/analyze`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ market, account_margin_usd: marginUsd, risk_tolerance: riskTolerance }),
      });
      if (!r.ok) {
        const body = await r.json().catch(() => ({}));
        throw new Error(body?.detail || `Request failed (${r.status})`);
      }
      const data: AnalyzeResult = await r.json();
      setResult(data);
    } catch (e: any) {
      setError(e?.message || 'Analysis failed — try again');
    } finally {
      setLoading(false);
    }
  };

  // Auto-analyze on market or risk change
  useEffect(() => {
    if (!autoAnalyze) return;
    runAnalysis();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [market, riskTolerance, autoAnalyze]);

  const selectedMarket = MARKET_OPTIONS.find(m => m.id === market)!;
  const rsi = result?.indicators.rsi14;
  const rsiLabel = rsi == null ? null : rsi >= 70 ? 'Overbought' : rsi <= 30 ? 'Oversold' : 'Neutral';
  const rsiColor = rsi == null ? C.TEXT3 : rsi >= 70 ? C.RED : rsi <= 30 ? C.GREEN : C.TEXT2;

  return (
    <div style={{ background: C.SURF, border: `1px solid ${C.BDR2}`, borderRadius: 16, padding: '16px', marginBottom: 14 }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4, flexWrap: 'wrap' }}>
        <span style={{ fontSize: '1.1rem' }}>🤖</span>
        <div style={{ fontWeight: 800, fontSize: '0.9rem', color: C.TEXT }}>AI Trade Analysis</div>
        <span style={{ fontSize: '0.58rem', color: C.TEAL, background: C.TEALG, padding: '2px 7px', borderRadius: 6, fontWeight: 700 }}>4H</span>
        <span style={{ fontSize: '0.58rem', color: C.TEAL, background: C.TEALG, padding: '2px 7px', borderRadius: 6, fontWeight: 700 }}>{RISK_LABEL[riskTolerance]} RISK</span>
        <span style={{ fontSize: '0.58rem', color: C.GOLD, background: `${C.GOLD}15`, padding: '2px 7px', borderRadius: 6, fontWeight: 700 }}>2:1 R:R</span>
      </div>
      <div style={{ fontSize: '0.68rem', color: C.TEXT3, marginBottom: 14, lineHeight: 1.4 }}>
        4H price history + Trend-Following / Breakout / ORB signal scan. Take-profit = 2× stop-loss distance. Auto-refreshes when you switch asset. Not financial advice.
      </div>

      {/* Market + risk selectors */}
      <div style={{ display: 'flex', gap: 6, marginBottom: 10, flexWrap: 'wrap' }}>
        {MARKET_OPTIONS.map(m => (
          <button key={m.id} onClick={() => { setMarket(m.id); setResult(null); }} style={{ padding: '7px 12px', borderRadius: 100, background: market === m.id ? `${m.color}20` : 'rgba(255,255,255,.05)', border: `1px solid ${market === m.id ? m.color + '50' : C.BDR2}`, color: market === m.id ? m.color : C.TEXT3, fontSize: '0.7rem', fontWeight: 700, cursor: 'pointer' }}>{m.icon} {m.sym}</button>
        ))}
      </div>
      <div style={{ display: 'flex', gap: 6, marginBottom: 14 }}>
        {(['conservative', 'moderate', 'aggressive'] as RiskTolerance[]).map(r => (
          <button key={r} onClick={() => setRiskTolerance(r)} style={{ flex: 1, padding: '7px 0', borderRadius: 9, background: riskTolerance === r ? C.TEALG : 'transparent', border: `1px solid ${riskTolerance === r ? C.TEAL + '50' : C.BDR2}`, color: riskTolerance === r ? C.TEAL : C.TEXT3, fontWeight: 700, fontSize: '0.68rem', cursor: 'pointer', textTransform: 'capitalize' }}>{r} ({RISK_LABEL[r]})</button>
        ))}
      </div>

      <button onClick={runAnalysis} disabled={loading} style={{ width: '100%', padding: '12px', borderRadius: 12, background: C.TEAL, border: 'none', color: '#04120A', fontWeight: 800, fontSize: '0.85rem', cursor: 'pointer', opacity: loading ? 0.6 : 1, display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 8, marginBottom: 14 }}>
        {loading ? (<><span style={{ width: 14, height: 14, border: '2px solid #04120A30', borderTop: '2px solid #04120A', borderRadius: '50%', animation: 'spin 1s linear infinite', display: 'inline-block' }} /> Analyzing {selectedMarket.sym} (4H)…</>) : `Re-analyze ${selectedMarket.sym} (4H)`}
      </button>

      {error && (
        <div style={{ padding: '10px 12px', borderRadius: 10, background: `${C.RED}12`, border: `1px solid ${C.RED}28`, color: C.RED, fontSize: '0.75rem', marginBottom: 14, lineHeight: 1.4 }}>{error}</div>
      )}

      {result && (
        <div>
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', gap: 8, marginBottom: 14 }}>
            <div style={{ background: C.SURF2, borderRadius: 10, padding: '8px 10px' }}>
              <div style={{ fontSize: '0.55rem', color: C.TEXT3, textTransform: 'uppercase', letterSpacing: '.05em' }}>Price</div>
              <div style={{ fontSize: '0.8rem', fontWeight: 700, color: C.TEXT }}>{fmtUSD(result.indicators.price)}</div>
            </div>
            <div style={{ background: C.SURF2, borderRadius: 10, padding: '8px 10px' }}>
              <div style={{ fontSize: '0.55rem', color: C.TEXT3, textTransform: 'uppercase', letterSpacing: '.05em' }}>~24h Change</div>
              <div style={{ fontSize: '0.8rem', fontWeight: 700, color: (result.indicators.change_24h_pct ?? 0) >= 0 ? C.GREEN : C.RED }}>{result.indicators.change_24h_pct != null ? `${result.indicators.change_24h_pct >= 0 ? '+' : ''}${result.indicators.change_24h_pct.toFixed(2)}%` : '—'}</div>
            </div>
            <div style={{ background: C.SURF2, borderRadius: 10, padding: '8px 10px' }}>
              <div style={{ fontSize: '0.55rem', color: C.TEXT3, textTransform: 'uppercase', letterSpacing: '.05em' }}>RSI (14)</div>
              <div style={{ fontSize: '0.8rem', fontWeight: 700, color: rsiColor }}>{rsi != null ? rsi.toFixed(1) : '—'} {rsiLabel && <span style={{ fontSize: '0.55rem' }}>({rsiLabel})</span>}</div>
            </div>
            <div style={{ background: C.SURF2, borderRadius: 10, padding: '8px 10px' }}>
              <div style={{ fontSize: '0.55rem', color: C.TEXT3, textTransform: 'uppercase', letterSpacing: '.05em' }}>SMA 20 / 50</div>
              <div style={{ fontSize: '0.75rem', fontWeight: 700, color: C.TEXT }}>{result.indicators.sma20 ? fmtUSD(result.indicators.sma20) : '—'} / {result.indicators.sma50 ? fmtUSD(result.indicators.sma50) : '—'}</div>
            </div>
            <div style={{ background: C.SURF2, borderRadius: 10, padding: '8px 10px' }}>
              <div style={{ fontSize: '0.55rem', color: C.TEXT3, textTransform: 'uppercase', letterSpacing: '.05em' }}>30d Range</div>
              <div style={{ fontSize: '0.72rem', fontWeight: 700, color: C.TEXT }}>{fmtUSD(result.indicators.low_30d)} – {fmtUSD(result.indicators.high_30d)}</div>
            </div>
            <div style={{ background: C.SURF2, borderRadius: 10, padding: '8px 10px' }}>
              <div style={{ fontSize: '0.55rem', color: C.TEXT3, textTransform: 'uppercase', letterSpacing: '.05em' }}>ATR (14)</div>
              <div style={{ fontSize: '0.8rem', fontWeight: 700, color: C.TEXT }}>{result.indicators.atr14 ? fmtUSD(result.indicators.atr14) : '—'}</div>
            </div>
          </div>

          <div style={{ background: C.SURF2, borderRadius: 12, padding: '14px', whiteSpace: 'pre-wrap', fontSize: '0.78rem', color: C.TEXT2, lineHeight: 1.6, marginBottom: 10 }}>
            {result.analysis}
          </div>

          {onApplySuggestion && (
            <div style={{ display: 'flex', gap: 8 }}>
              <button onClick={() => onApplySuggestion('long')} style={{ flex: 1, padding: '9px 0', borderRadius: 10, background: `${C.GREEN}18`, border: `1px solid ${C.GREEN}40`, color: C.GREEN, fontWeight: 700, fontSize: '0.72rem', cursor: 'pointer' }}>Set up as Long</button>
              <button onClick={() => onApplySuggestion('short')} style={{ flex: 1, padding: '9px 0', borderRadius: 10, background: `${C.RED}18`, border: `1px solid ${C.RED}40`, color: C.RED, fontWeight: 700, fontSize: '0.72rem', cursor: 'pointer' }}>Set up as Short</button>
            </div>
          )}

          <div style={{ fontSize: '0.6rem', color: C.TEXT3, marginTop: 10, textAlign: 'center' }}>
            Model: {result.model} · Data: 4H bars (CoinGecko) + AI · Not financial advice
          </div>
        </div>
      )}
    </div>
  );
};

export default AIAdvisorPanel;