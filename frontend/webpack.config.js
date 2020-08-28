const path = require('path');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');
const CopyWebpackPlugin = require('copy-webpack-plugin');

const distPath = path.resolve(__dirname, "../dist");
module.exports = (env, argv) => {
  return {
    devServer: {
      contentBase: distPath,
      compress: argv.mode === 'production',
      port: 8000,
      historyApiFallback: {
        index: 'index.html'
      },
      proxy: {
        '/api': {
          target: {
            host: "127.0.0.1",
            protocol: 'http:',
            port: 3030
          },
        }
      }
    },
    entry: './bootstrap.js',
    output: {
      path: distPath,
      filename: "chatr.js",
      webassemblyModuleFilename: "chatr.wasm"
    },
    module: {
      rules: [
        {
          test: /\.s?[ac]ss$/i,
          use: [
            'style-loader',
            'css-loader',
            'sass-loader',
          ],
        },
      ],
    },
    plugins: [
      new CopyWebpackPlugin([
        { from: './static', to: distPath }
      ]),
      new WasmPackPlugin({
        crateDirectory: ".",
        extraArgs: "--no-typescript",
      })
    ],
    watch: argv.mode !== 'production'
  };
};
