const path = require('path');
const { SwcJsMinimizerRspackPlugin } = require('@rspack/core');

/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
  mode: 'production',
  entry: './src/index.jsx',
  output: {
    filename: 'bundle.js',
    path: path.resolve(__dirname, 'dist'),
  },
  optimization: {
    minimize: true,
    minimizer: [new SwcJsMinimizerRspackPlugin({
      extractComments: {
        condition: /MARK_EXPRESSIONS/,
        filename: "[file].comments.txt",
      },
    })],
  },
  module: {
    rules: [
      {
        test: /\.m?[jt]sx?$/,
        exclude: /(node_modules)/,
        use: {
          loader: 'builtin:swc-loader',
          options: {
            jsc: {
              parser: {
                syntax: "typescript",
                tsx: true
              },
              experimental: {
                plugins: [
                  [
                    'swc-plugin-mark-expressions',
                    {
                      title: "MARK_EXPRESSIONS",
                      functions: ["markFnA", "markFnB", "markFnC"],
                      methods: {
                          window: ["markWindowFnA", "markWindowFnB", "markWindowFnC"],
                          this: ["markThisFnA", "markThisFnB", "markThisFnC"],
                          obj: ["markObjFnA", "markObjFnB", "markObjFnC"]
                      },
                      dynamicImports: ["shouldMark"]
                    }
                  ]
                ]
              }
            }
          }
        }
      }
    ]
  }
};
