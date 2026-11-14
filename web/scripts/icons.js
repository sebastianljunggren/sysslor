// Renders the PNG icons that browsers and iOS need from the SVGs in static/.
// The output in static/icons/ is generated, so it is gitignored.
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { Resvg } from '@resvg/resvg-js';

const icons = [
  { svg: 'icon.svg', png: 'icon-192.png', size: 192 },
  { svg: 'icon.svg', png: 'icon-512.png', size: 512 },
  { svg: 'icon-maskable.svg', png: 'maskable-512.png', size: 512 },
  // iOS fills transparent corners with black, so this uses the full-bleed version.
  { svg: 'icon-maskable.svg', png: 'apple-touch-icon.png', size: 180 },
];

const dir = new URL('../static/', import.meta.url);
mkdirSync(new URL('icons/', dir), { recursive: true });
for (const { svg, png, size } of icons) {
  const source = readFileSync(new URL(svg, dir));
  const image = new Resvg(source, { fitTo: { mode: 'width', value: size } }).render();
  writeFileSync(new URL(`icons/${png}`, dir), image.asPng());
}
