# Tests GPU layout requirements

This example shows us what requirements exist with respect to layout. For example, we may need to 
manually adjust the alignment of a structure's member due to the one of GPU layout requirements.

All tests are composed of error cases and workaround cases about a specific requirement. To see
what error message will be shown, you need to uncomment the error case.

Please see the [src/lib.rs](src/lib.rs) for more detail.

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

## How to clean up after build or test

```sh
# Remove "dist" and "pkg" directories
npm run clean

# Remove all directories produced by build and test processes
npm run clean-all
```

