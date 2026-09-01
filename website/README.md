Source for the Korigio public download page at [m-wri.github.io/servioo](https://m-wri.github.io/servioo/).

The live site lives in the public repo [M-WRI/servioo](https://github.com/M-WRI/servioo). On each `v*` tag, the Release workflow:

1. Uploads installers to a public GitHub Release
2. Runs `scripts/build-download-page.mjs` against those releases
3. Commits the baked `index.html` (latest download cards plus a previous-versions list)

`SERVIOO_RELEASES_TOKEN` must be set on this private repo or that job fails.

```bash
npm run website:build   # regenerate website/index.html from public releases
```
