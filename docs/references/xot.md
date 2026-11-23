# XOT openings

XOT varies play by starting from a roughly balanced position reached by eight
legal moves. Matthias Berg and Borja Moreno compiled the lists using Edax and
Ntest at levels 16 and 20. The large list contains 10,784 sequences evaluated
from −2 to +2 after move eight. The small list contains 3,623 sequences
evaluated from −1 to +1. [About XOT](https://berg.earthlingz.de/xot/aboutxot.php?lang=en)

| Ruleset | Original file | SHA-256 |
| --- | --- | --- |
| `standard-xot-large` | [openingslarge.txt](https://berg.earthlingz.de/xot/downloads/openingslarge.txt) | `ccbeb0ed7b7ae23de20013a3992b9a6ede77610bbb5cacda82ccbaedaab72530` |
| `standard-xot-small` | [openingssmall.txt](https://berg.earthlingz.de/xot/downloads/openingssmall.txt) | `8a441b077b1093af7b0d1cb55fa5a0d3d1700ce200a1b2d5b6e28d7b17139580` |

These hashes identify the standalone files retrieved on 2026-11-24. Each
LF-terminated row consists of eight lowercase algebraic coordinates with no
separator. For example, `f5` is column f, row 5. File order is significant for
seeded selection.

The source invites downloads to play XOT and displays a copyright notice.
The About and Download pages provide no explicit redistribution license.
The source repository and its public build outputs therefore exclude both the
raw lists and artifacts embedding them. [XOT downloads](https://berg.earthlingz.de/xot/download.php?lang=en)

## Prepare an operator bundle

```sh
tools/release-packager/fetch-xot.sh .tmp/xot
XOT_OPENINGS_DIR="$PWD/.tmp/xot" AI_ARENA_DIR=/path/to/ai-arena \
  RELEASE_VERSION=v1.2.0 make verify-release-artifacts
```

Use the same game ZIP when registering all three competition rulesets. Select
`standard-xot-large` for the default XOT competition or
`standard-xot-small` for the narrower evaluation range.
