const path = require('path');
const TerserPlugin = require('terser-webpack-plugin');

module.exports = {
  mode: 'production',
  entry: './src/index.js',
  output: {
    filename: 'bundle.js',
    path: path.resolve(__dirname, 'dist'),
  },
  optimization: {
    minimize: true,
    minimizer: [new TerserPlugin({
      terserOptions: {
        format: {
          comments: /marked/
        }
      }
    })]
  },
  module: {
    rules: [
      {
        test: /\.m?js$/,
        exclude: /(node_modules)/,
        use: {
          loader: 'swc-loader',
          options: {
            jsc: {
              experimental: {
                plugins: [
                  [
                    'swc-plugin-mark-expressions',
                    { 
                      title: 'marked',
                      functions: ['markedFunction', 'anotherMarkedFunction'],
                      objects: ['window', 'global'],
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