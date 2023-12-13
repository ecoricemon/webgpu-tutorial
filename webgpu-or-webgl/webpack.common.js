const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");

const dist = path.resolve(__dirname, "dist");

module.exports = {
  entry: {
    index: "./js/index.js"
  },
  output: {
    path: dist,
    filename: "[name].js"
  },
  plugins: [
    new CopyPlugin({
      patterns: [
        path.resolve(__dirname, "static")
      ],
    }),
  ],
  experiments: {
    asyncWebAssembly: true,
  },
  performance: {
    maxAssetSize: 500000,
    maxEntrypointSize: 500000,
  }
};
