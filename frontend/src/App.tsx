import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { DICTS, LANG_LABELS, SUPPORTED, type Lang } from './i18n';

// Initial language was resolved synchronously in index.html (no-flash). Read it
// back; fall back to English if anything went wrong.
function initialLang(): Lang {
  const g = (window.__DN7_LANG__ || 'en') as Lang;
  return SUPPORTED.includes(g) ? g : 'en';
}

const GITHUB_URL = 'https://github.com/Digital-Network-7/DN7-Website';

// Reveal-on-scroll wrapper: fades + lifts its children into view the first time
// they enter the viewport. `delay` staggers grouped items.
function Reveal({
  children,
  delay = 0,
  className = '',
}: {
  children: React.ReactNode;
  delay?: number;
  className?: string;
}) {
  const ref = useRef<HTMLDivElement | null>(null);
  const [shown, setShown] = useState(false);
  useEffect(() => {
    const node = ref.current;
    if (!node) return;
    if (typeof IntersectionObserver === 'undefined') {
      setShown(true);
      return;
    }
    const io = new IntersectionObserver(
      (entries) => {
        entries.forEach((e) => {
          if (e.isIntersecting) {
            setShown(true);
            io.disconnect();
          }
        });
      },
      { threshold: 0.14, rootMargin: '0px 0px -8% 0px' },
    );
    io.observe(node);
    return () => io.disconnect();
  }, []);
  return (
    <div
      ref={ref}
      className={`reveal${shown ? ' in' : ''} ${className}`.trim()}
      style={{ transitionDelay: `${delay}ms` }}
    >
      {children}
    </div>
  );
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
        <Download t={t} />
        <Hero t={t} />
        <Products t={t} />
        <Product t={t} />
        <OpenSource t={t} />
        <Features t={t} />
      </main>
      <Footer t={t} />
    </div>
  );
}

function Backdrop() {
  return (
    <div className="backdrop" aria-hidden="true">
      <div className="grid" />
    </div>
  );
}

type T = (typeof DICTS)[Lang];

function Header({ t, lang, onLang }: { t: T; lang: Lang; onLang: (l: Lang) => void }) {
  const [open, setOpen] = useState(false);
  return (
    <header className="hdr-bar">
      <div className="hdr">
        <a className="brand" href="#download">
          <img src="/logo.svg" alt="Digital Network 7" />
          <span>Digital Network 7</span>
        </a>
        <nav className="nav">
          <a href="#download">{t.nav.download}</a>
          <a href="#product">{t.nav.product}</a>
          <a href="#features">{t.nav.features}</a>
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
      </div>
    </header>
  );
}

function Hero({ t }: { t: T }) {
  return (
    <section className="hero" id="overview">
      <div className="hero-inner">
        <span className="badge enter" style={{ animationDelay: '60ms' }}>{t.hero.badge}</span>
        <h1 className="hero-title enter" style={{ animationDelay: '140ms' }}>{t.hero.title}</h1>
        <p className="hero-sub enter" style={{ animationDelay: '240ms' }}>{t.hero.subtitle}</p>
        <div className="hero-cta enter" style={{ animationDelay: '340ms' }}>
          <a className="btn" href="#download">{t.hero.ctaDownload}</a>
          <a className="btn ghost" href="#product">{t.hero.ctaLearn}</a>
        </div>
      </div>
      <div className="hero-art enter-art" style={{ animationDelay: '200ms' }}>
        <PanelGlyph />
      </div>
    </section>
  );
}

function Product({ t }: { t: T }) {
  return (
    <section className="product" id="product">
      <Reveal className="prod-card">
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
      </Reveal>
    </section>
  );
}

function Products({ t }: { t: T }) {
  return (
    <section className="products" id="products">
      <Reveal className="sec-title">{t.products.title}</Reveal>
      {t.products.subtitle && <Reveal className="sec-sub" delay={60}>{t.products.subtitle}</Reveal>}
      <div className="prod-grid">
        {t.products.items.map((p, i) => {
          const stable = p.status === 'stable';
          return (
            <Reveal key={i} className={`prodc${stable ? ' live' : ''}`} delay={i * 90}>
              <div className="prodc-top">
                <div className="prodc-ic"><ProductIcon i={i} /></div>
                <span className={`statusbadge ${stable ? 'on' : 'soon'}`}>
                  <span className="sdot" />{stable ? t.products.stable : t.products.soon}
                </span>
              </div>
              <h3>{p.name}</h3>
              <p>{p.desc}</p>
              <a className="prodc-link" href={stable ? '#product' : '#opensource'}>
                {t.products.view} <span className="arr">→</span>
              </a>
            </Reveal>
          );
        })}
      </div>
    </section>
  );
}

function OpenSource({ t }: { t: T }) {
  return (
    <section className="opensource" id="opensource">
      <Reveal className="os-card">
        <div className="os-ic"><GitIcon /></div>
        <h2>{t.opensource.title}</h2>
        <p>{t.opensource.desc}</p>
        <a className="btn ghost os-btn" href={GITHUB_URL} target="_blank" rel="noreferrer">
          <GitIcon /> {t.opensource.cta}
        </a>
        <code className="os-url">{GITHUB_URL}</code>
      </Reveal>
    </section>
  );
}

function Features({ t }: { t: T }) {
  return (
    <section className="features" id="features">
      <Reveal className="sec-title">{t.features.title}</Reveal>
      {t.features.subtitle && <Reveal className="sec-sub" delay={60}>{t.features.subtitle}</Reveal>}
      <div className="feat-grid">
        {t.features.items.map((f, i) => (
          <Reveal key={i} className="feat" delay={i * 80}>
            <div className="feat-ic"><FeatIcon i={i} /></div>
            <h3>{f.title}</h3>
            <p>{f.desc}</p>
          </Reveal>
        ))}
      </div>
    </section>
  );
}

function Download({ t }: { t: T }) {
  const [copied, setCopied] = useState(false);
  const cmd = t.download.oneLine;

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
      <Reveal className="sec-title">{t.download.title}</Reveal>
      {t.download.subtitle && <Reveal className="sec-sub" delay={60}>{t.download.subtitle}</Reveal>}

      <Reveal className="cmd" delay={120}>
        <code>{cmd}</code>
        <button className="copybtn" onClick={copy}>{copied ? t.download.copied : t.download.copy}</button>
      </Reveal>

      <Reveal className="term" delay={160}>
        <FakeTerminal cmd={cmd} />
      </Reveal>

      <Reveal className="dl-note" delay={220}>{t.download.note}</Reveal>
    </section>
  );
}

// A decorative terminal that types out the install command and a simulated,
// brand-neutral install transcript. Purely illustrative — no real version data.
function FakeTerminal({ cmd }: { cmd: string }) {
  const ref = useRef<HTMLDivElement | null>(null);
  const [typed, setTyped] = useState('');
  const [revealed, setRevealed] = useState(0); // how many transcript lines shown
  const [done, setDone] = useState(false);
  const started = useRef(false);

  const transcript = useMemo(
    () => [
      '[DN7] detecting architecture … x86_64',
      '[DN7] downloading DN7 Panel ▕████████████████▏ 100%',
      '[DN7] verifying signature … ok',
      '[DN7] installing …',
      '[DN7] starting DN7 Panel ✓',
      '',
      '  console   →  http://203.0.113.17:1080',
      '            →  http://192.168.1.20:1080',
      '  username  →  admin',
      '  password  →  k7P2-xQ9m-Lf3a',
      '  status    →  running',
      '',
    ],
    [],
  );

  useEffect(() => {
    const node = ref.current;
    if (!node || typeof IntersectionObserver === 'undefined') {
      setTyped(cmd);
      setRevealed(transcript.length);
      setDone(true);
      return;
    }
    const io = new IntersectionObserver(
      (entries) => {
        entries.forEach((e) => {
          if (e.isIntersecting && !started.current) {
            started.current = true;
            runAnimation();
            io.disconnect();
          }
        });
      },
      { threshold: 0.3 },
    );
    io.observe(node);

    const timers: number[] = [];
    function runAnimation() {
      let i = 0;
      const typeTimer = window.setInterval(() => {
        i++;
        setTyped(cmd.slice(0, i));
        if (i >= cmd.length) {
          window.clearInterval(typeTimer);
          transcript.forEach((_, idx) => {
            timers.push(
              window.setTimeout(() => {
                setRevealed(idx + 1);
                if (idx === transcript.length - 1) setDone(true);
              }, 360 + idx * 300),
            );
          });
        }
      }, 42);
      timers.push(typeTimer);
    }
    return () => {
      io.disconnect();
      timers.forEach((id) => window.clearTimeout(id));
    };
  }, [cmd, transcript]);

  return (
    <div className="term-box" ref={ref}>
      <div className="term-head">
        <span className="td r" />
        <span className="td y" />
        <span className="td g" />
        <span className="term-host">dn7@example</span>
      </div>
      <div className="term-body">
        <div className="term-line">
          <span className="prompt">$</span>
          <span className="typed">{typed}</span>
          {typed.length < cmd.length && <span className="caret" />}
        </div>
        {/* All lines are always rendered so the box height is fixed; un-revealed
            ones stay invisible (but occupy space) → no layout shift / jump. */}
        {transcript.map((l, i) => (
          <div className={`term-out${i < revealed ? ' show' : ''}`} key={i}>{l || '\u00a0'}</div>
        ))}
        <div className="term-line" style={{ visibility: done ? 'visible' : 'hidden' }}>
          <span className="prompt">$</span>
          <span className="caret" />
        </div>
      </div>
    </div>
  );
}

function Footer({ t }: { t: T }) {
  const year = new Date().getFullYear();
  return (
    <footer className="ftr">
      <div className="ftr-top">
        <a className="brand sm" href="#download">
          <img src="/logo.svg" alt="Digital Network 7" />
          <span>Digital Network 7</span>
        </a>
        <p className="ftr-tag">{t.footer.tagline}</p>
      </div>
      <div className="ftr-bot">
        <span>© 2025–{year} DN7.cn {t.footer.rights}</span>
        <a href="https://beian.miit.gov.cn/" target="_blank" rel="noreferrer">{t.footer.beian}</a>
        <span className="ftr-friends">
          {t.footer.friends}:{' '}
          <a href="https://linux.do" target="_blank" rel="noreferrer">LINUX DO</a>
        </span>
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

function GitIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
      <path d="M12 2C6.48 2 2 6.58 2 12.25c0 4.53 2.87 8.37 6.84 9.73.5.09.68-.22.68-.49 0-.24-.01-.88-.01-1.73-2.78.62-3.37-1.21-3.37-1.21-.46-1.18-1.11-1.49-1.11-1.49-.91-.64.07-.62.07-.62 1 .07 1.53 1.05 1.53 1.05.89 1.56 2.34 1.11 2.91.85.09-.66.35-1.11.63-1.36-2.22-.26-4.56-1.14-4.56-5.07 0-1.12.39-2.03 1.03-2.75-.1-.26-.45-1.3.1-2.71 0 0 .84-.27 2.75 1.05A9.36 9.36 0 0 1 12 7.07c.85 0 1.71.12 2.51.34 1.91-1.32 2.75-1.05 2.75-1.05.55 1.41.2 2.45.1 2.71.64.72 1.03 1.63 1.03 2.75 0 3.94-2.34 4.81-4.57 5.06.36.32.68.94.68 1.9 0 1.37-.01 2.48-.01 2.82 0 .27.18.59.69.49A10.02 10.02 0 0 0 22 12.25C22 6.58 17.52 2 12 2z" />
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

function ProductIcon({ i }: { i: number }) {
  // 0: Panel (dashboard), 1: Drive (stacked clouds), 2: CDN (globe + edges)
  const icons = [
    <g key="p"><rect x="3" y="4" width="18" height="16" rx="2.5" /><path d="M3 9h18M7 14h5M7 17h8" /></g>,
    <g key="d"><path d="M7 10a4 4 0 0 1 7.5-1.5A3.5 3.5 0 0 1 18 15H8a3.5 3.5 0 0 1-1-6.9" /><path d="M9 19h9" opacity="0.6" /></g>,
    <g key="c"><circle cx="12" cy="12" r="8.5" /><path d="M3.5 12h17M12 3.5c2.4 2.3 2.4 14.7 0 17M12 3.5c-2.4 2.3-2.4 14.7 0 17" /></g>,
  ];
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round">
      {icons[i % icons.length]}
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
