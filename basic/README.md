## How to install npm packages

```sh
npm install
```

## How to build in **debug** mode

```sh
npm run build
```

## Hot to run webpack dev server

```sh
npm start
```

## How to build in **release** mode

```sh
npm run build-release
```

## How to test

```sh
# It uses chromedriver basically
npm test
```

## How to test on other browsers

```sh
# Firefox
wasm-pack test --firefox --headless --workspace

# Safari
wasm-pack test --safari --headless --workspace
```

## How to clean up after build or test

```sh
# Remove "dist" and "pkg" directories
npm run clean

# Remove all directories produced by build and test processes
npm run clean-all
```
