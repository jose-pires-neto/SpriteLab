# Reproducing the REA investigation

Used REA npm 6.4.0, Node 26.5.0 and macOS arm64. No global MCP registration was needed. The repository skill was read at commit `43738bec579fc69bab084bffa964916d43d43ed1`. Public Aseprite source reference: `14fe117e853bd642672e72018166239506060cd4`.

1. Obtain the official [macOS trial](https://www.aseprite.org/trial/) (investigated version 1.3.18.6), review its terms and mount it read-only. The demo cannot save; do not bypass that limitation.
2. Install REA in a private working directory: `npm install --prefix ./rea-runtime rea-agents@6.4.0 --ignore-scripts`.
3. Run the private CLI against the mounted bundle:

```sh
node rea-runtime/node_modules/rea-agents/scripts/rea.mjs inspect-artifact /absolute/mount/Aseprite.app --json > bundle.json
node rea-runtime/node_modules/rea-agents/scripts/rea.mjs inspect-plist /absolute/mount/Aseprite.app --json > plist.json
```

4. Write a caller-selected scenario, using its actual absolute executable/work paths:

```json
{"executable":"/absolute/mount/Aseprite.app/Contents/MacOS/aseprite","arguments":["--version"],"working_directory":"/absolute/work","timeout_ms":10000,"settle_ms":100}
```

5. `node rea-runtime/node_modules/rea-agents/scripts/rea.mjs capture-process scenario.json --json > version.json`. A `--help` scenario changes only the explicit arguments. Preserve error envelopes, partial observations and cleanup limitations. Do not treat CLI exit code alone as complete REA evidence.
6. Unmount the read-only image when finished. No Aseprite code/assets are imported into SpriteLab.

The local investigation needed unsandboxed mounting/native process tools after concrete sandbox failures. `--help` yielded observed exit/output but failed REA cleanup ownership verification on macOS; its error and partial capture are retained in this repository's evidence directory. `--version`, bundle inventory and plist inspection completed successfully. No Hopper/Ghidra/IDA provider was installed or invoked.

Aseprite source analysis uses ordinary repository reads, following the REA skill. See the evidence map for the exact source locations. There is no claim of binary decompilation or full black-box reconstruction.
