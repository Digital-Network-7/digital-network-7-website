import { useCallback, useEffect, useMemo, useState } from 'react';
import { DICTS, LANG_LABELS, SUPPORTED, type Lang } from './i18n';

// Initial language was resolved synchronously in index.html (no-flash). Read it
// back; fall back to English if anything went wrong.
function initialLang(): Lang {
  const g = (window.__DN7_LANG__ || 'en') as Lang;
  return SUPPORTED.includes(g) ? g : 'en';
}

interface Manifest {
  product: string;
  version: string;
  sizes: Record<string, number>;
  sha256: Record<string, string>;
  downloads: Record<string, string>;
  install: string;
}

function fmtBytes(n?: number): string {
  if (!n || n <= 0) return '—';
  const u = ['B', 'KB', 'MB', 'GB'];
  let i = 0;
  let v = n;
  while (v >= 1024 && i < u.length - 1) {
    v /= 1024;
    i++;
  }
  return `${v.toFixed(v < 10 && i > 0 ? 1 : 0)} ${u[i]}`;
}

export default function App() {
  const [lang, setLang] = useState<Lang>(initialLang);
  const t = DICTS[lang];

  // Persist + reflect language choice. Never auto-overrides a saved choice.
  const changeLang = useCallback((l: Lang) => {
    setLang(l);
    try {
      localStorage.setItem('dn7_lang', l);
    } catch {
      /* ignore */
    }
    document.documentElement.lang = l;
  }, []);

  return (
    <div className="page">
      <Backdrop />
      <Header t={t} lang={lang} onLang={changeLang} />
      <main>
        <Hero t={t} />
        <Layers t={t} />
        <Product t={t} />
        <Features t={t} />
        <Download t={t} />
      </main>
      <Footer t={t} />
    </div>
  );
}

function Backdrop() {
  return (
    <div className="backdrop" aria-hidden="true">
      <div className="grid" />
      <span className="orb o1" />
      <span className="orb o2" />
      <span className="orb o3" />
    </div>
  );
}

type T = (typeof DICTS)[Lang];

function Header({ t, lang, onLang }: { t: T; lang: Lang; onLang: (l: Lang) => void }) {
  const [open, setOpen] = useState(false);
  return (
    <header className="hdr">
      <a className="brand" href="#top">
        <img src="/logo.png" alt="Digital Network 7" />
        <span>Digital Network 7</span>
      </a>
      <nav className="nav">
        <a href="#product">{t.nav.product}</a>
        <a href="#features">{t.nav.features}</a>
        <a href="#download">{t.nav.download}</a>
      </nav>
      <div className="langpick">
        <button className="langbtn" onClick={() => setOpen((v) => !v)} onBlur={() => setTimeout(() => setOpen(false), 150)}>
          <GlobeIcon />
          <span>{LANG_LABELS[lang]}</span>
        </button>
        {open && (
          <div className="langpop">
            {SUPPORTED.map((l) => (
              <button key={l} className={l === lang ? 'on' : ''} onMouseDown={(e) => { e.preventDefault(); onLang(l); setOpen(false); }}>
                {LANG_LABELS[l]}
              </button>
            ))}
          </div>
        )}
      </div>
    </header>
  );
}

function Hero({ t }: { t: T }) {
  return (
    <section className="hero" id="top">
      <div className="hero-inner">
        <span className="badge">{t.hero.badge}</span>
        <h1 className="hero-title">{t.hero.title}</h1>
        <p className="hero-sub">{t.hero.subtitle}</p>
        <div className="hero-cta">
          <a className="btn" href="#download">{t.hero.ctaDownload}</a>
          <a className="btn ghost" href="#product">{t.hero.ctaLearn}</a>
        </div>
      </div>
      <div className="hero-art">
        <NetworkArt />
      </div>
    </section>
  );
}

function Layers({ t }: { t: T }) {
  return (
    <section className="layers" id="layers">
      <h2 className="sec-title">{t.layers.title}</h2>
      <p className="sec-sub">{t.layers.subtitle}</p>
      <div className="layer-stack">
        {t.layers.items.map((name, i) => (
          <div className="layer" key={i} style={{ ['--i' as string]: String(i) }}>
            <span className="layer-no">L{7 - i}</span>
            <span className="layer-name">{name}</span>
            <span className="layer-line" />
          </div>
        ))}
      </div>
    </section>
  );
}

function Product({ t }: { t: T }) {
  return (
    <section className="product" id="product">
      <div className="prod-card">
        <div className="prod-text">
          <span className="tag">{t.product.tag}</span>
          <h2>{t.product.title}</h2>
          <p>{t.product.desc}</p>
          <ul className="checks">
            {t.product.points.map((p, i) => (
              <li key={i}><CheckIcon />{p}</li>
            ))}
          </ul>
        </div>
        <div className="prod-visual">
          <PanelGlyph />
        </div>
      </div>
    </section>
  );
}

function Features({ t }: { t: T }) {
  return (
    <section className="features" id="features">
      <h2 className="sec-title">{t.features.title}</h2>
      <p className="sec-sub">{t.features.subtitle}</p>
      <div className="feat-grid">
        {t.features.items.map((f, i) => (
          <div className="feat" key={i}>
            <div className="feat-ic"><FeatIcon i={i} /></div>
            <h3>{f.title}</h3>
            <p>{f.desc}</p>
          </div>
        ))}
      </div>
    </section>
  );
}

function Download({ t }: { t: T }) {
  const [copied, setCopied] = useState(false);
  const [manifest, setManifest] = useState<Manifest | null>(null);
  const [state, setState] = useState<'loading' | 'ok' | 'err'>('loading');

  useEffect(() => {
    let alive = true;
    fetch('/api/panel/latest')
      .then((r) => r.json())
      .then((b) => {
        if (!alive) return;
        if (b && b.ok && b.data) {
          setManifest(b.data as Manifest);
          setState('ok');
        } else {
          setState('err');
        }
      })
      .catch(() => alive && setState('err'));
    return () => {
      alive = false;
    };
  }, []);

  const cmd = useMemo(() => manifest?.install || t.download.oneLine, [manifest, t]);

  const copy = () => {
    navigator.clipboard?.writeText(cmd).then(
      () => {
        setCopied(true);
        setTimeout(() => setCopied(false), 1600);
      },
      () => {},
    );
  };

  return (
    <section className="download" id="download">
      <h2 className="sec-title">{t.download.title}</h2>
      <p className="sec-sub">{t.download.subtitle}</p>

      <div className="cmd">
        <code>{cmd}</code>
        <button className="copybtn" onClick={copy}>{copied ? t.download.copied : t.download.copy}</button>
      </div>

      <div className="rel">
        <div className="rel-ver">
          <span className="rel-label">{t.download.version}</span>
          {state === 'loading' && <span className="rel-val dim">{t.download.loading}</span>}
          {state === 'err' && <span className="rel-val dim">{t.download.unavailable}</span>}
          {state === 'ok' && manifest && <span className="rel-val">v{manifest.version || '—'}</span>}
        </div>

        {state === 'ok' && manifest && (
          <div className="rel-bins">
            <div className="rel-bins-h">{t.download.binaries}</div>
            <table>
              <thead>
                <tr>
                  <th>{t.download.arch}</th>
                  <th>{t.download.size}</th>
                  <th />
                </tr>
              </thead>
              <tbody>
                {['x86_64', 'arm64'].map((arch) => (
                  <tr key={arch}>
                    <td className="mono">{arch}</td>
                    <td className="mono dim">{fmtBytes(manifest.sizes?.[arch])}</td>
                    <td>
                      <a className="dlbtn" href={manifest.downloads?.[arch] || `/api/panel/download?arch=${arch}`}>
                        ↓
                      </a>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
      <p className="dl-note">{t.download.note}</p>
    </section>
  );
}

function Footer({ t }: { t: T }) {
  const year = new Date().getFullYear();
  return (
    <footer className="ftr">
      <div className="ftr-top">
        <a className="brand sm" href="#top">
          <img src="/logo.png" alt="Digital Network 7" />
          <span>Digital Network 7</span>
        </a>
        <p className="ftr-tag">{t.footer.tagline}</p>
      </div>
      <div className="ftr-bot">
        <span>© 2025–{year} dn7.cn {t.footer.rights}</span>
        <a href="https://beian.miit.gov.cn/" target="_blank" rel="noreferrer">{t.footer.beian}</a>
      </div>
    </footer>
  );
}

/* ---- inline SVG bits ---- */

function GlobeIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round">
      <circle cx="12" cy="12" r="9" />
      <path d="M3 12h18M12 3c2.5 2.5 2.5 15 0 18M12 3c-2.5 2.5-2.5 15 0 18" />
    </svg>
  );
}

function CheckIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.4" strokeLinecap="round" strokeLinejoin="round">
      <path d="M5 12.5l4.5 4.5L19 7" />
    </svg>
  );
}

function NetworkArt() {
  // A stylized seven-node network: concentric rings + linked nodes, animated.
  const nodes = Array.from({ length: 7 }, (_, i) => {
    const a = (i / 7) * Math.PI * 2 - Math.PI / 2;
    return { x: 150 + Math.cos(a) * 110, y: 150 + Math.sin(a) * 110, i };
  });
  return (
    <svg className="net" viewBox="0 0 300 300" fill="none">
      <circle cx="150" cy="150" r="110" className="net-ring" />
      <circle cx="150" cy="150" r="70" className="net-ring" />
      <circle cx="150" cy="150" r="32" className="net-ring" />
      {nodes.map((n) => (
        <line key={`l${n.i}`} x1="150" y1="150" x2={n.x} y2={n.y} className="net-link" style={{ ['--d' as string]: String(n.i) }} />
      ))}
      {nodes.map((n) => (
        <line key={`r${n.i}`} x1={n.x} y1={n.y} x2={nodes[(n.i + 1) % 7].x} y2={nodes[(n.i + 1) % 7].y} className="net-link faint" />
      ))}
      {nodes.map((n) => (
        <circle key={`n${n.i}`} cx={n.x} cy={n.y} r="7" className="net-node" style={{ ['--d' as string]: String(n.i) }} />
      ))}
      <circle cx="150" cy="150" r="11" className="net-core" />
    </svg>
  );
}

function PanelGlyph() {
  return (
    <svg viewBox="0 0 240 180" fill="none" className="panel-glyph">
      <rect x="8" y="8" width="224" height="164" rx="14" className="pg-frame" />
      <rect x="8" y="8" width="224" height="34" rx="14" className="pg-bar" />
      <circle cx="28" cy="25" r="4" className="pg-dot" />
      <circle cx="44" cy="25" r="4" className="pg-dot" />
      <circle cx="60" cy="25" r="4" className="pg-dot" />
      <rect x="26" y="60" width="80" height="46" rx="8" className="pg-cell" />
      <rect x="118" y="60" width="96" height="46" rx="8" className="pg-cell" />
      <rect x="26" y="118" width="188" height="34" rx="8" className="pg-cell dim" />
      <path d="M34 92c10-18 22-18 32 0s22 18 32 0" className="pg-spark" />
    </svg>
  );
}

function FeatIcon({ i }: { i: number }) {
  const paths = [
    <path key="a" d="M12 3v9m0 0l4-4m-4 4L8 8M5 21h14" />,
    <path key="b" d="M4 7h16v10H4zM4 11h16" />,
    <path key="c" d="M12 3l8 4v6c0 4-3.5 7-8 8-4.5-1-8-4-8-8V7z" />,
    <path key="d" d="M3 17l5-5 4 4 8-9" />,
    <path key="e" d="M4 5h7v7H4zM13 5h7v4h-7zM13 12h7v7h-7zM4 14h7v5H4z" />,
    <path key="f" d="M12 3l8 4v6c0 4-3.5 7-8 8-4.5-1-8-4-8-8V7zM9.5 12l2 2 3.5-4" />,
  ];
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round">
      {paths[i % paths.length]}
    </svg>
  );
}
