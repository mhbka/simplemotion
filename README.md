## Native library and CLI

The reusable parser and motion model live in `simplemotion-lib`. Test or inspect
motion files without building WebAssembly:

```sh
cargo test -p simplemotion-lib
cargo run -p simplemotion-cli -- examples/sd002.json
```

The root package remains the WebAssembly wrapper and depends on `simplemotion-lib`,
so browser bindings can be added later without moving the core code again.

## Web project: install

```sh
npm install
```

## How to run in debug mode

```sh
# Builds the project and opens it in a new browser tab. Auto-reloads when the project changes.
npm start
```

## How to build in release mode

```sh
# Builds the project and places it into the `dist` folder.
npm run build
```

## How to run unit tests

```sh
# Runs all native library and CLI tests, then browser tests in Firefox.
npm test -- --firefox

# Runs tests in Chrome
npm test -- --chrome

# Runs tests in Safari
npm test -- --safari
```

## What does each file do?

* `Cargo.toml` contains the standard Rust metadata. You put your Rust dependencies in here. You must change this file with your details (name, description, version, authors, categories)

* `package.json` contains the standard npm metadata. You put your JavaScript dependencies in here. You must change this file with your details (author, name, version)

* `webpack.config.js` contains the Webpack configuration. You shouldn't need to change this, unless you have very special needs.

* The `js` folder contains your JavaScript code (`index.js` is used to hook everything into Webpack, you don't need to change it).

* `simplemotion-lib` contains the reusable native Rust parser and motion model.

* `simplemotion-cli` contains the native command-line inspector.

* The `src` folder contains the WebAssembly wrapper, which can expose the native library to JavaScript.

* The `static` folder contains any files that you want copied as-is into the final build. It contains an `index.html` file which loads the `index.js` file.

* Each native project keeps its tests in its own `tests` folder.
