import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

const config: Config = {
  title: 'bevy_cef',
  tagline: 'Chromium Embedded Framework integration for Bevy',
  favicon: 'img/favicon.ico',

  future: {
    v4: true,
  },

  url: 'https://nickkro.github.io',
  baseUrl: '/bevy_cef/',

  organizationName: 'nickkro',
  projectName: 'bevy_cef',

  onBrokenLinks: 'throw',

  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          routeBasePath: '/',
          editUrl: 'https://github.com/nickkro/bevy_cef/tree/main/docs/website/',
        },
        blog: false,
        theme: {
          customCss: './src/css/custom.css',
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    colorMode: {
      respectPrefersColorScheme: true,
    },
    navbar: {
      title: 'bevy_cef',
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'docsSidebar',
          position: 'left',
          label: 'Docs',
        },
        {
          href: 'https://github.com/nickkro/bevy_cef',
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Docs',
          items: [
            {
              label: 'Getting Started',
              to: '/getting-started/your-first-webview',
            },
            {
              label: 'Reference',
              to: '/reference/components-and-events',
            },
          ],
        },
        {
          title: 'More',
          items: [
            {
              label: 'GitHub',
              href: 'https://github.com/nickkro/bevy_cef',
            },
            {
              label: 'crates.io',
              href: 'https://crates.io/crates/bevy_cef',
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} bevy_cef contributors. Built with Docusaurus.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ['rust', 'toml', 'powershell', 'bash'],
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
