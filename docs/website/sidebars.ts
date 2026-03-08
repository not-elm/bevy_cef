import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

/**
 * Sidebar structure for bevy_cef documentation.
 *
 * Growth conventions:
 * - Communication: IPC and messaging patterns (JS Emit, Host Emit, BRP)
 * - Guides: Bevy-side features and configuration (one page per feature)
 * - Reference: Lookup tables and API docs (add rows for new components)
 *
 * Introduce subcategories in Guides when it exceeds ~10-12 items.
 */
const sidebars: SidebarsConfig = {
  docsSidebar: [
    'intro',
    'installation',
    {
      type: 'category',
      label: 'Getting Started',
      items: [
        'getting-started/your-first-webview',
        'getting-started/talking-to-your-webview',
      ],
    },
    {
      type: 'category',
      label: 'Communication',
      link: {
        type: 'doc',
        id: 'communication/index',
      },
      items: [
        'communication/host-emit',
        'communication/brp',
      ],
    },
    {
      type: 'category',
      label: 'Guides',
      items: [
        'guides/local-assets',
        'guides/navigation',
        'guides/devtools',
        'guides/sprite-rendering',
        'guides/custom-materials',
        'guides/preload-scripts',
        'guides/extensions',
        'guides/zoom-and-audio',
      ],
    },
    'concepts',
    {
      type: 'category',
      label: 'Reference',
      items: [
        'reference/components-and-events',
        'reference/plugin-configuration',
        'reference/javascript-api',
        'reference/version-compatibility',
      ],
    },
  ],
};

export default sidebars;
