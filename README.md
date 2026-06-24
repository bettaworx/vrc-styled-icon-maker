# vrc-styled-icon-maker
Create an icon to be used in the VRChat Expression Menu

![alt text](https://raw.githubusercontent.com/bettaworx/vrc-styled-icon-maker/refs/heads/main/assets/Thumbnail.png "Preview")

## How To Use
You can use the tool by accessing the [web app](https://vsim.bettaworx.net) or using the [CLI app](https://github.com/bettaworx/vrc-styled-icon-maker/releases/latest).

### Web App

Access the hosted version at [vsim.bettaworx.net](https://vsim.bettaworx.net) — no installation required.

![How to use the Web App](https://raw.githubusercontent.com/bettaworx/vrc-styled-icon-maker/refs/heads/main/assets/HowToUse_webApp.png "How to use the Web App")

To run locally:

**Prerequisites:** [Node.js](https://nodejs.org/), [pnpm](https://pnpm.io/), [Rust](https://www.rust-lang.org/tools/install), [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)

```sh
# Build the WASM package
wasm-pack build --target web --scope bettaworx crates/wasm

# Start the dev server
cd web
pnpm install
pnpm dev
```

### CLI App

Download the latest binary for your platform from [GitHub Releases](https://github.com/bettaworx/vrc-styled-icon-maker/releases/latest), or build from source:

```sh
cargo build --release -p app
```

**Usage:**

```
vrc-styled-icon-maker <input.svg> output.png
vrc-styled-icon-maker -i <input> -o <output> [-s] [-c] [-r params]
vrc-styled-icon-maker -i <glob_pattern> -o <output_dir>
```

| Flag | Description |
|------|-------------|
| `-i` | Input file, glob pattern, or directory (can specify multiple) |
| `-o` | Output file or directory |
| `-s` | Apply VRC icon style (gradient background, shadow, padding) |
| `-c` | PNG-to-SVG vectorization params (PNG inputs are always vectorized; use `-c` to customize) |
| `-r` | Renderer params (e.g. `width=512,height=512`) |

**Examples:**

```sh
# Simple SVG to PNG conversion
vrc-styled-icon-maker icon.svg icon.png

# Apply VRC style
vrc-styled-icon-maker -i icon.svg -o styled.png -s

# Batch process a directory
vrc-styled-icon-maker -i ./icons/ -o ./output/
```

## Q&A

#### Q. How does the web app work?
A. It runs on the your web browser (WASM), which means your data is processed locally in your browser and not sent to any server.

#### Q. Can I use png icon?
A. Yes, but it will be converted to a vector image internally for processing and the result may not be as good. So I'd say SVG is better, which gives you better results.

## License
This project is licensed under the [MIT License](https://github.com/bettaworx/vrc-styled-icon-maker/blob/main/LICENSE).

## Acknowledgment
This repository includes a material based on Phosphor Icons (Copyright (c) 2020 Phosphor Icons), distributed under the MIT License.

```md
MIT License

Copyright (c) 2020 Phosphor Icons

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
