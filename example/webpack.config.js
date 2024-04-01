const path = require('path');
const TerserPlugin = require('terser-webpack-plugin');

module.exports = {
  mode: 'production',
  entry: './src/index.jsx',
  output: {
    filename: 'bundle.js',
    path: path.resolve(__dirname, 'dist'),
  },
  optimization: {
    minimize: true,
    minimizer: [new TerserPlugin({
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
          loader: 'swc-loader',
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
}