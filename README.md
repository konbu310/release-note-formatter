# release-note-formatter
githubが自動生成するMarkdownのリリースノートをScrapbox用に変換するよ

## GitHub Pages用ビルド

`cargo-make` を入れてから以下で `./docs` に出力できる。

```sh
cargo install cargo-make --locked
cargo make pages
```
