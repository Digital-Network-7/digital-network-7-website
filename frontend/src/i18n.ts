// i18n dictionaries for the DN7 website. Four languages today; adding another
// is just one more entry here plus its code in SUPPORTED.

export type Lang = 'zh-CN' | 'zh-TW' | 'en' | 'ja';

export const SUPPORTED: Lang[] = ['zh-CN', 'zh-TW', 'en', 'ja'];

export const LANG_LABELS: Record<Lang, string> = {
  'zh-CN': '简体中文',
  'zh-TW': '繁體中文',
  en: 'English',
  ja: '日本語',
};

export interface Dict {
  nav: { product: string; features: string; download: string; docs: string };
  hero: {
    badge: string;
    title: string;
    subtitle: string;
    ctaDownload: string;
    ctaLearn: string;
  };
  layers: { title: string; subtitle: string; items: string[] };
  product: {
    tag: string;
    title: string;
    desc: string;
    points: string[];
  };
  features: { title: string; subtitle: string; items: { title: string; desc: string }[] };
  download: {
    title: string;
    subtitle: string;
    oneLine: string;
    copy: string;
    copied: string;
    version: string;
    loading: string;
    unavailable: string;
    binaries: string;
    arch: string;
    size: string;
    note: string;
  };
  footer: { rights: string; beian: string; tagline: string };
}

const en: Dict = {
  nav: { product: 'Product', features: 'Features', download: 'Download', docs: 'Docs' },
  hero: {
    badge: 'Next-generation distributed network infrastructure',
    title: 'Digital Network 7',
    subtitle:
      'A seven-layer network platform built for scalable systems, developer-first tooling, and cloud-native operations. Composable from the wire up.',
    ctaDownload: 'Get DN7 Panel',
    ctaLearn: 'Explore the stack',
  },
  layers: {
    title: 'Seven layers, one network',
    subtitle:
      'DN7 is structured as a clean seven-layer model — each layer composable, observable, and built to scale independently.',
    items: [
      'Physical & transport fabric',
      'Secure overlay & NAT traversal',
      'Service mesh & routing',
      'Control plane & orchestration',
      'Observability & metrics',
      'Management & automation',
      'Experience & interface',
    ],
  },
  product: {
    tag: 'Flagship product',
    title: 'DN7 Panel',
    desc:
      'A single static binary that turns any Linux host into a fully managed node — monitoring, web terminal, container, web-server and database management, file transfer, all over a NAT-friendly outbound connection.',
    points: [
      'Real-time metrics: CPU, memory, disk, network throughput',
      'Browser terminal and per-container shells',
      'Docker, web-server and database management built in',
      'Secure file transfer and scan-to-login',
      'One binary, no dependencies — works behind NAT',
    ],
  },
  features: {
    title: 'Built for operators',
    subtitle: 'Everything you need to run fleets of machines, nothing you do not.',
    items: [
      { title: 'Outbound-only', desc: 'Agents dial home, so intranet and NAT hosts work with no inbound ports.' },
      { title: 'Single binary', desc: 'Pure static build, no runtime dependencies, deploys in one command.' },
      { title: 'Staged rollout', desc: 'Updates roll out gradually and are globally rate-limited.' },
      { title: 'Observable', desc: 'Live metrics and process insight across every layer of the stack.' },
      { title: 'Composable', desc: 'Each layer is independent — adopt only what you need.' },
      { title: 'Secure by design', desc: 'Encrypted channels, signed releases, least-privilege access.' },
    ],
  },
  download: {
    title: 'Install in one line',
    subtitle: 'Paste this on any Linux host. Future installs and updates flow through dn7.cn.',
    oneLine: 'curl -fsSL https://dn7.cn/start.sh | sh',
    copy: 'Copy',
    copied: 'Copied',
    version: 'Latest version',
    loading: 'Checking latest version…',
    unavailable: 'Version info unavailable right now',
    binaries: 'Direct downloads',
    arch: 'Architecture',
    size: 'Size',
    note: 'Linux x86_64 and arm64. Updates are delivered automatically.',
  },
  footer: {
    rights: 'All rights reserved.',
    beian: '沪ICP备2026021336号',
    tagline: 'Digital Network 7 — composable infrastructure for distributed systems.',
  },
};

const zhCN: Dict = {
  nav: { product: '产品', features: '特性', download: '下载', docs: '文档' },
  hero: {
    badge: '新一代分布式网络基础设施',
    title: 'Digital Network 7',
    subtitle:
      '面向可扩展系统、开发者优先工具与云原生运维打造的七层网络平台。从底层连接开始，皆可组合。',
    ctaDownload: '获取 DN7 Panel',
    ctaLearn: '了解架构',
  },
  layers: {
    title: '七层网络，一体协同',
    subtitle: 'DN7 以清晰的七层模型构建 —— 每一层均可组合、可观测，并能独立扩展。',
    items: [
      '物理与传输底座',
      '安全叠加网络与 NAT 穿透',
      '服务网格与路由',
      '控制平面与编排',
      '可观测性与指标',
      '管理与自动化',
      '体验与交互界面',
    ],
  },
  product: {
    tag: '旗舰产品',
    title: 'DN7 Panel',
    desc:
      '一个静态二进制文件，即可把任意 Linux 主机变成全托管节点 —— 监控、网页终端、容器/网站/数据库管理、文件传输，全部通过对 NAT 友好的出站连接完成。',
    points: [
      '实时指标：CPU、内存、磁盘、网络吞吐',
      '浏览器终端与容器内 Shell',
      '内置 Docker、网站与数据库管理',
      '安全文件传输与扫码登录',
      '单文件、零依赖 —— 可在 NAT 后运行',
    ],
  },
  features: {
    title: '为运维而生',
    subtitle: '管理成规模的机器所需的一切，不多不少。',
    items: [
      { title: '纯出站连接', desc: 'Agent 主动外连，内网与 NAT 主机无需开放任何入站端口。' },
      { title: '单一二进制', desc: '纯静态构建，无运行时依赖，一条命令完成部署。' },
      { title: '灰度发布', desc: '更新逐步推送，并在全局范围内限速。' },
      { title: '可观测', desc: '跨越每一层的实时指标与进程洞察。' },
      { title: '可组合', desc: '每一层都相互独立 —— 按需采用。' },
      { title: '安全设计', desc: '加密信道、签名发布、最小权限访问。' },
    ],
  },
  download: {
    title: '一行命令安装',
    subtitle: '在任意 Linux 主机粘贴执行。未来的安装与更新都将通过 dn7.cn。',
    oneLine: 'curl -fsSL https://dn7.cn/start.sh | sh',
    copy: '复制',
    copied: '已复制',
    version: '最新版本',
    loading: '正在获取最新版本…',
    unavailable: '暂时无法获取版本信息',
    binaries: '直接下载',
    arch: '架构',
    size: '大小',
    note: '支持 Linux x86_64 与 arm64，更新将自动下发。',
  },
  footer: {
    rights: '版权所有。',
    beian: '沪ICP备2026021336号',
    tagline: 'Digital Network 7 —— 面向分布式系统的可组合基础设施。',
  },
};

const zhTW: Dict = {
  nav: { product: '產品', features: '特性', download: '下載', docs: '文件' },
  hero: {
    badge: '新一代分散式網路基礎設施',
    title: 'Digital Network 7',
    subtitle:
      '為可擴展系統、開發者優先工具與雲原生維運打造的七層網路平台。從底層連線開始，皆可組合。',
    ctaDownload: '取得 DN7 Panel',
    ctaLearn: '了解架構',
  },
  layers: {
    title: '七層網路，一體協同',
    subtitle: 'DN7 以清晰的七層模型構建 —— 每一層皆可組合、可觀測，並能獨立擴展。',
    items: [
      '實體與傳輸底層',
      '安全疊加網路與 NAT 穿透',
      '服務網格與路由',
      '控制平面與編排',
      '可觀測性與指標',
      '管理與自動化',
      '體驗與互動介面',
    ],
  },
  product: {
    tag: '旗艦產品',
    title: 'DN7 Panel',
    desc:
      '一個靜態二進位檔，即可把任意 Linux 主機變成全託管節點 —— 監控、網頁終端、容器/網站/資料庫管理、檔案傳輸，全部透過對 NAT 友善的出站連線完成。',
    points: [
      '即時指標：CPU、記憶體、磁碟、網路吞吐',
      '瀏覽器終端與容器內 Shell',
      '內建 Docker、網站與資料庫管理',
      '安全檔案傳輸與掃碼登入',
      '單檔、零依賴 —— 可在 NAT 後運行',
    ],
  },
  features: {
    title: '為維運而生',
    subtitle: '管理成規模機器所需的一切，不多不少。',
    items: [
      { title: '純出站連線', desc: 'Agent 主動外連，內網與 NAT 主機無需開放任何入站連接埠。' },
      { title: '單一二進位', desc: '純靜態建置，無執行時相依，一條命令完成部署。' },
      { title: '灰度發布', desc: '更新逐步推送，並在全域範圍內限速。' },
      { title: '可觀測', desc: '跨越每一層的即時指標與行程洞察。' },
      { title: '可組合', desc: '每一層皆相互獨立 —— 按需採用。' },
      { title: '安全設計', desc: '加密通道、簽章發布、最小權限存取。' },
    ],
  },
  download: {
    title: '一行命令安裝',
    subtitle: '在任意 Linux 主機貼上執行。未來的安裝與更新都將透過 dn7.cn。',
    oneLine: 'curl -fsSL https://dn7.cn/start.sh | sh',
    copy: '複製',
    copied: '已複製',
    version: '最新版本',
    loading: '正在取得最新版本…',
    unavailable: '暫時無法取得版本資訊',
    binaries: '直接下載',
    arch: '架構',
    size: '大小',
    note: '支援 Linux x86_64 與 arm64，更新將自動下發。',
  },
  footer: {
    rights: '版權所有。',
    beian: '滬ICP備2026021336號',
    tagline: 'Digital Network 7 —— 面向分散式系統的可組合基礎設施。',
  },
};

const ja: Dict = {
  nav: { product: '製品', features: '特長', download: 'ダウンロード', docs: 'ドキュメント' },
  hero: {
    badge: '次世代の分散ネットワーク基盤',
    title: 'Digital Network 7',
    subtitle:
      'スケーラブルなシステム、開発者ファーストのツール、クラウドネイティブ運用のために設計された七層ネットワーク基盤。接続の最下層から、すべてが組み合わせ可能。',
    ctaDownload: 'DN7 Panel を入手',
    ctaLearn: 'アーキテクチャを見る',
  },
  layers: {
    title: '七つの層、ひとつのネットワーク',
    subtitle:
      'DN7 は明快な七層モデルで構成され、各層は組み合わせ可能・観測可能で、独立してスケールします。',
    items: [
      '物理・トランスポート基盤',
      'セキュアオーバーレイと NAT 越え',
      'サービスメッシュとルーティング',
      'コントロールプレーンとオーケストレーション',
      '可観測性とメトリクス',
      '管理と自動化',
      '体験とインターフェース',
    ],
  },
  product: {
    tag: '主力製品',
    title: 'DN7 Panel',
    desc:
      '単一の静的バイナリで、あらゆる Linux ホストを完全管理ノードに。監視、ウェブ端末、コンテナ／ウェブサーバー／データベース管理、ファイル転送を、NAT に優しいアウトバウンド接続で実現します。',
    points: [
      'リアルタイム指標：CPU・メモリ・ディスク・ネットワーク',
      'ブラウザ端末とコンテナ内シェル',
      'Docker・ウェブサーバー・データベース管理を内蔵',
      '安全なファイル転送とスキャンログイン',
      '単一バイナリ・依存ゼロ —— NAT 越しでも動作',
    ],
  },
  features: {
    title: '運用者のために',
    subtitle: '大量のマシンを運用するために必要なものすべてを、過不足なく。',
    items: [
      { title: 'アウトバウンドのみ', desc: 'エージェントが自ら接続するため、社内網や NAT 配下でも受信ポート不要。' },
      { title: '単一バイナリ', desc: '純粋な静的ビルド、ランタイム依存なし、ワンコマンドで配備。' },
      { title: '段階的ロールアウト', desc: '更新は徐々に配信され、全体でレート制限されます。' },
      { title: '可観測', desc: 'スタックの全層にわたるライブ指標とプロセス把握。' },
      { title: '組み合わせ可能', desc: '各層は独立 —— 必要なものだけを採用。' },
      { title: '設計から安全', desc: '暗号化チャネル、署名済みリリース、最小権限アクセス。' },
    ],
  },
  download: {
    title: 'ワンラインでインストール',
    subtitle: '任意の Linux ホストに貼り付けて実行。今後のインストールと更新は dn7.cn 経由になります。',
    oneLine: 'curl -fsSL https://dn7.cn/start.sh | sh',
    copy: 'コピー',
    copied: 'コピーしました',
    version: '最新バージョン',
    loading: '最新バージョンを取得中…',
    unavailable: '現在バージョン情報を取得できません',
    binaries: '直接ダウンロード',
    arch: 'アーキテクチャ',
    size: 'サイズ',
    note: 'Linux x86_64 と arm64 に対応。更新は自動で配信されます。',
  },
  footer: {
    rights: 'All rights reserved.',
    beian: '沪ICP备2026021336号',
    tagline: 'Digital Network 7 — 分散システムのための組み合わせ可能な基盤。',
  },
};

export const DICTS: Record<Lang, Dict> = {
  'zh-CN': zhCN,
  'zh-TW': zhTW,
  en,
  ja,
};
