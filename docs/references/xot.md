# XOT openings

XOT starts from a roughly even board after eight legal moves. Matthias Berg
and Borja Moreno built both lists with Edax and Ntest at levels 16 and 20.

After move eight, the 10,784 large-list rows score from −2 to +2. The 3,623
small-list rows score from −1 to +1. [About XOT](https://berg.earthlingz.de/xot/aboutxot.php?lang=en)

| Ruleset | Original file | SHA-256 |
| --- | --- | --- |
| `standard-xot-large` | [openingslarge.txt](https://berg.earthlingz.de/xot/downloads/openingslarge.txt) | `ccbeb0ed7b7ae23de20013a3992b9a6ede77610bbb5cacda82ccbaedaab72530` |
| `standard-xot-small` | [openingssmall.txt](https://berg.earthlingz.de/xot/downloads/openingssmall.txt) | `8a441b077b1093af7b0d1cb55fa5a0d3d1700ce200a1b2d5b6e28d7b17139580` |

The repository stores the original files retrieved on 2026-11-24 in
`games/reversi/data/xot/`. Each row ends in LF
and holds eight lowercase coordinates with no separator. A coordinate such as
`f5` means column f, row 5. Seeded selection uses the original file order.

The source offers the lists for play and displays a copyright notice. Its
About and Download pages provide no explicit redistribution license.
[XOT downloads](https://berg.earthlingz.de/xot/download.php?lang=en)

## Retrieve and verify the originals

From the repository root, download a fresh copy into a scratch directory and
compare it with the tracked files.

```sh
tools/release-packager/fetch-xot.sh .tmp/xot-source
cmp .tmp/xot-source/openingslarge.txt games/reversi/data/xot/openingslarge.txt
cmp .tmp/xot-source/openingssmall.txt games/reversi/data/xot/openingssmall.txt
(cd games/reversi/data/xot && sha256sum openingslarge.txt openingssmall.txt)
```

The hashes must match the table above. The build checks the hashes, row
counts, uniqueness, and legality before embedding both files into the WASM.

## Build and verify a bundle

```sh
AI_ARENA_DIR=/path/to/ai-arena RELEASE_VERSION=v1.2.0 make verify-release-artifacts
```

Use the same game ZIP when registering all three competition rulesets. Select
`standard-xot-large` for the default XOT competition or
`standard-xot-small` for the narrower evaluation range.
