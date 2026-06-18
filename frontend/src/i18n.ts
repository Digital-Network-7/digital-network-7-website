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
  product: {
    tag: string;
    title: string;
    desc: string;
    points: string[];
  };
  products: {
    title: string;
    subtitle: string;
    stable: string;
    soon: string;
    view: string;
    items: { name: string; desc: string; status: 'stable' | 'soon' }[];
  };
  opensource: { title: string; desc: string; cta: string };
  features: { title: string; subtitle: string; items: { title: string; desc: string }[] };
  download: {
    title: string;
    subtitle: string;
    oneLine: string;
    copy: string;
    copied: string;
    note: string;
  };
  footer: { rights: string; beian: string; tagline: string; friends: string };
}

const en: Dict = {
  nav: { product: 'Product', features: 'Features', download: 'Download', docs: 'Docs' },
  hero: {
    badge: 'Open-source Linux server panel',
    title: 'Digital Network 7',
    subtitle: 'Install DN7 Panel on a Linux host and manage terminals, files, containers, websites and databases from one web console.',
    ctaDownload: 'Get DN7 Panel',
    ctaLearn: 'Learn more',
  },
  product: {
    tag: 'Flagship',
    title: 'DN7 Panel',
    desc: 'One binary turns any Linux host into a managed node.',
    points: [
      'Live metrics & web terminal',
      'Docker, web server & database',
      'Secure file transfer',
      'Single binary, no dependencies',
    ],
  },
  products: {
    title: 'The DN7 suite',
    subtitle: 'Open-source building blocks for distributed infrastructure.',
    stable: 'Stable',
    soon: 'In development',
    view: 'Learn more',
    items: [
      { name: 'DN7 Panel', desc: 'One binary turns any Linux host into a managed node.', status: 'stable' },
      { name: 'DN7 Drive', desc: 'Unify Google Drive, iCloud, OneDrive, Tencent, Quark, Baidu and more in one place.', status: 'soon' },
      { name: 'DN7 CDN', desc: 'A toolkit that helps you build and run your own CDN.', status: 'soon' },
    ],
  },
  opensource: {
    title: 'Fully open source',
    desc: 'Every line is public and auditable. Read the code, verify the builds, contribute.',
    cta: 'View on GitHub',
  },
  features: {
    title: 'Built for operators',
    subtitle: '',
    items: [
      { title: 'Single binary', desc: 'Static build, one command.' },
      { title: 'Auto updates', desc: 'New versions roll out automatically.' },
      { title: 'Observable', desc: 'Live host metrics in the browser.' },
      { title: 'Service management', desc: 'Handle common server workflows in one place.' },
      { title: 'Extensible', desc: 'A plugin model for new capabilities.' },
      { title: 'Secure by design', desc: 'Encrypted channels, signed releases.' },
    ],
  },
  download: {
    title: 'Install in one line',
    subtitle: 'Run this on a Linux host. The installer chooses x86_64 or arm64 automatically.',
    oneLine: 'curl -fsSL https://dn7.cn/start.sh | sh',
    copy: 'Copy',
    copied: 'Copied',
    note: 'Linux x86_64 & arm64.',
  },
  footer: {
    rights: 'All rights reserved.',
    beian: '沪ICP备2026021336号',
    tagline: 'Open-source tools for Linux servers and self-hosted infrastructure.',
    friends: 'Friends',
  },
};

const zhCN: Dict = {
  nav: { product: '产品', features: '特性', download: '下载', docs: '文档' },
  hero: {
    badge: '开源 Linux 服务器面板',
    title: 'Digital Network 7',
    subtitle: '在 Linux 主机上一行安装 DN7 Panel，通过网页控制台管理终端、文件、容器、网站与数据库。',
    ctaDownload: '获取 DN7 Panel',
    ctaLearn: '了解更多',
  },
  product: {
    tag: '旗舰',
    title: 'DN7 Panel',
    desc: '一个二进制，让任意 Linux 主机成为受管节点。',
    points: ['实时指标与网页终端', 'Docker、网站与数据库', '安全文件传输', '单文件，零依赖'],
  },
  products: {
    title: 'DN7 套件',
    subtitle: '面向分布式基础设施的开源构件。',
    stable: '稳定版',
    soon: '开发中',
    view: '了解更多',
    items: [
      { name: 'DN7 Panel', desc: '一个二进制，让任意 Linux 主机成为受管节点。', status: 'stable' },
      { name: 'DN7 Drive', desc: '聚合谷歌、苹果、OneDrive、腾讯、夸克、百度等多家网盘于一处。', status: 'soon' },
      { name: 'DN7 CDN', desc: '帮助你搭建并运营自己的 CDN 的工具。', status: 'soon' },
    ],
  },
  opensource: {
    title: '完全开源',
    desc: '每一行代码都公开、可审计。阅读源码、验证构建、参与共建。',
    cta: '在 GitHub 查看',
  },
  features: {
    title: '为运维而生',
    subtitle: '',
    items: [
      { title: '单一二进制', desc: '静态构建，一条命令。' },
      { title: '自动更新', desc: '新版本自动下发。' },
      { title: '可观测', desc: '在浏览器查看主机实时指标。' },
      { title: '服务管理', desc: '把常见服务器操作集中到一处。' },
      { title: '可扩展', desc: '插件化，能力随需生长。' },
      { title: '安全设计', desc: '加密信道，签名发布。' },
    ],
  },
  download: {
    title: '一行命令安装',
    subtitle: '在 Linux 主机上执行，安装器会自动选择 x86_64 或 arm64。',
    oneLine: 'curl -fsSL https://dn7.cn/start.sh | sh',
    copy: '复制',
    copied: '已复制',
    note: '支持 Linux x86_64 与 arm64。',
  },
  footer: {
    rights: '版权所有。',
    beian: '沪ICP备2026021336号',
    tagline: '面向 Linux 服务器与自托管基础设施的开源工具。',
    friends: '友情链接',
  },
};

const zhTW: Dict = {
  nav: { product: '產品', features: '特性', download: '下載', docs: '文件' },
  hero: {
    badge: '開源 Linux 伺服器面板',
    title: 'Digital Network 7',
    subtitle: '在 Linux 主機上一行安裝 DN7 Panel，透過網頁控制台管理終端、檔案、容器、網站與資料庫。',
    ctaDownload: '取得 DN7 Panel',
    ctaLearn: '了解更多',
  },
  product: {
    tag: '旗艦',
    title: 'DN7 Panel',
    desc: '一個二進位，讓任意 Linux 主機成為受管節點。',
    points: ['即時指標與網頁終端', 'Docker、網站與資料庫', '安全檔案傳輸', '單檔，零依賴'],
  },
  products: {
    title: 'DN7 套件',
    subtitle: '面向分散式基礎設施的開源構件。',
    stable: '穩定版',
    soon: '開發中',
    view: '了解更多',
    items: [
      { name: 'DN7 Panel', desc: '一個二進位，讓任意 Linux 主機成為受管節點。', status: 'stable' },
      { name: 'DN7 Drive', desc: '聚合 Google、Apple、OneDrive、騰訊、夸克、百度等多家網盤於一處。', status: 'soon' },
      { name: 'DN7 CDN', desc: '協助你搭建並營運自己的 CDN 的工具。', status: 'soon' },
    ],
  },
  opensource: {
    title: '完全開源',
    desc: '每一行程式碼都公開、可稽核。閱讀原始碼、驗證建置、參與共建。',
    cta: '在 GitHub 查看',
  },
  features: {
    title: '為維運而生',
    subtitle: '',
    items: [
      { title: '單一二進位', desc: '靜態建置，一條命令。' },
      { title: '自動更新', desc: '新版本自動下發。' },
      { title: '可觀測', desc: '在瀏覽器查看主機即時指標。' },
      { title: '服務管理', desc: '把常見伺服器操作集中到一處。' },
      { title: '可擴展', desc: '外掛化，能力隨需成長。' },
      { title: '安全設計', desc: '加密通道，簽章發布。' },
    ],
  },
  download: {
    title: '一行命令安裝',
    subtitle: '在 Linux 主機上執行，安裝器會自動選擇 x86_64 或 arm64。',
    oneLine: 'curl -fsSL https://dn7.cn/start.sh | sh',
    copy: '複製',
    copied: '已複製',
    note: '支援 Linux x86_64 與 arm64。',
  },
  footer: {
    rights: '版權所有。',
    beian: '滬ICP備2026021336號',
    tagline: '面向 Linux 伺服器與自託管基礎設施的開源工具。',
    friends: '友情連結',
  },
};

const ja: Dict = {
  nav: { product: '製品', features: '特長', download: 'ダウンロード', docs: 'ドキュメント' },
  hero: {
    badge: 'オープンソース Linux サーバーパネル',
    title: 'Digital Network 7',
    subtitle: 'Linux ホストに DN7 Panel を導入し、端末、ファイル、コンテナ、サイト、データベースを Web コンソールで管理できます。',
    ctaDownload: 'DN7 Panel を入手',
    ctaLearn: '詳しく',
  },
  product: {
    tag: '主力',
    title: 'DN7 Panel',
    desc: '単一バイナリで、Linux ホストを管理ノードに。',
    points: ['リアルタイム指標と端末', 'Docker・ウェブ・DB', '安全なファイル転送', '単一バイナリ・依存ゼロ'],
  },
  products: {
    title: 'DN7 スイート',
    subtitle: '分散インフラのためのオープンソース部品。',
    stable: '安定版',
    soon: '開発中',
    view: '詳しく',
    items: [
      { name: 'DN7 Panel', desc: '単一バイナリで、Linux ホストを管理ノードに。', status: 'stable' },
      { name: 'DN7 Drive', desc: 'Google・Apple・OneDrive・Tencent・Quark・Baidu など複数のクラウドを一元化。', status: 'soon' },
      { name: 'DN7 CDN', desc: '自分の CDN を構築・運用するためのツール。', status: 'soon' },
    ],
  },
  opensource: {
    title: '完全オープンソース',
    desc: 'すべてのコードが公開され、監査可能。ソースを読み、ビルドを検証し、貢献できます。',
    cta: 'GitHub で見る',
  },
  features: {
    title: '運用者のために',
    subtitle: '',
    items: [
      { title: '単一バイナリ', desc: '静的ビルド、ワンコマンド。' },
      { title: '自動更新', desc: '新バージョンを自動配信。' },
      { title: '可観測', desc: 'ホストのライブ指標をブラウザで確認。' },
      { title: 'サービス管理', desc: '一般的なサーバー操作を一か所に集約。' },
      { title: '拡張可能', desc: 'プラグイン方式で機能を拡張。' },
      { title: '設計から安全', desc: '暗号化と署名済みリリース。' },
    ],
  },
  download: {
    title: 'ワンラインで導入',
    subtitle: 'Linux ホストで実行すると、x86_64 または arm64 を自動選択します。',
    oneLine: 'curl -fsSL https://dn7.cn/start.sh | sh',
    copy: 'コピー',
    copied: 'コピー完了',
    note: 'Linux x86_64 と arm64。',
  },
  footer: {
    rights: 'All rights reserved.',
    beian: '沪ICP备2026021336号',
    tagline: 'Linux サーバーとセルフホスト基盤のためのオープンソースツール。',
    friends: 'リンク',
  },
};

export const DICTS: Record<Lang, Dict> = {
  'zh-CN': zhCN,
  'zh-TW': zhTW,
  en,
  ja,
};
