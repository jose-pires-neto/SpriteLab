# Third-party notices

Original SpriteLab code is offered under MIT as a provisional permissive choice. Third-party packages retain their own notices/licenses; include their actual license texts when producing distributable builds.

The desktop shell uses Tauri 2 (MIT/Apache-2.0), React (MIT), Vite and their locked dependencies. [Collected installed dependency license texts](docs/DISTRIBUTION-LICENSES.txt) accompany the local app bundle.

The Rust prototype uses serde/serde_json/schemars/png/base64/gif/color_quant/weezl and their locked dependencies. [The dependency inventory](docs/rust-dependencies.json) records manifest license expressions for the installed lockfile. The MCP adapter uses official `@modelcontextprotocol/sdk` 1.32.1 (MIT) and the npm lockfile records its dependency versions. Installed package notices remain in their package directories. This inventory is not a completed binary-distribution license audit.

REA 6.4.0 was installed only in the investigation workspace. It is not a SpriteLab runtime dependency. Its original code is MIT; its third-party tooling has separate terms. REA evidence files are investigation outputs, not licensed copies of Aseprite.

Aseprite trial 1.3.18.6 and its public source were examined as references. Aseprite's [EULA](https://github.com/aseprite/aseprite/blob/14fe117e853bd642672e72018166239506060cd4/EULA.txt) restricts source compilation/modification, redistribution and reverse engineering subject to applicable-law exceptions. No Aseprite code, executable, palettes, icons, fonts or artwork is included in SpriteLab source. The demonstration mushroom was constructed by the original Python example using original pixel buffers and exported through SpriteLab. No binary decompilation/disassembly or license circumvention was performed.
