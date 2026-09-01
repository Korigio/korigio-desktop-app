Source for the Korigio public download page at [korigio.github.io/korigio-downloads](https://korigio.github.io/korigio-downloads/).

The live site lives in the public repo [Korigio/korigio-downloads](https://github.com/Korigio/korigio-downloads). On each `v*` tag, the Release workflow:

1. Uploads installers to a public GitHub Release
2. Runs `scripts/build-download-page.mjs` against those releases
3. Commits the baked `index.html` and `latest.json` (latest download cards plus a previous-versions list; `latest.json` is the update-check manifest next to the page)

`SERVIOO_RELEASES_TOKEN` must be set on this private repo or that job fails.

```bash
npm run website:build   # regenerate website/index.html and website/latest.json from public releases
```
