import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
  site: 'https://noplab.github.io',
  base: '/rk/docs',
  integrations: [
    starlight({
      title: 'RK',
      description: 'Documentation for the RK URDF Editor',
      social: {
        github: 'https://github.com/NOPLAB/rk',
      },
      sidebar: [
        {
          label: 'Getting Started',
          items: [
            { label: 'Installation', slug: 'getting-started/installation' },
            { label: 'Quick Start', slug: 'getting-started/quick-start' },
          ],
        },
        {
          label: 'Guides',
          autogenerate: { directory: 'guides' },
        },
        {
          label: 'Reference',
          autogenerate: { directory: 'reference' },
        },
      ],
      editLink: {
        baseUrl: 'https://github.com/NOPLAB/rk/edit/main/docs/',
      },
    }),
  ],
});
