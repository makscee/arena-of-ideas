# Share-card fonts (M4-7)

The fonts `server/src/mvp/share.ts` renders share cards with, so a card looks
the same on any host (resvg loads no system fonts). The client's own faces,
from Google Fonts (SIL Open Font License 1.1, the `OFL-*.txt` beside them):

- Chakra Petch Bold: titles (the client's `h1`, `h2`, buttons)
- Rajdhani SemiBold: names and text (the client's body)
- IBM Plex Mono Medium: numbers (PWR/HP, the address)
- Play Regular and Bold: any text with Cyrillic, which Chakra Petch and Rajdhani lack

Emoji are Twemoji's SVGs from the `@twemoji/svg` package (graphics CC-BY 4.0,
https://github.com/jdecked/twemoji).
