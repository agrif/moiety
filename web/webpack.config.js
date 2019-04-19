const path = require('path');
const webpack = require('webpack');
const HtmlWebpackPlugin = require('html-webpack-plugin');

module.exports = {
    mode: 'development',
    entry: './',
    output: {
        path: path.resolve(__dirname, 'out'),
        filename: 'bundle.js'
    },
    resolve: {
        alias: {
            moiety_web: path.resolve(__dirname, './pkg/moiety_web.js')
        }
    },
    module: {
        rules: [
            { test: /\.css$/, use: ['style-loader', 'css-loader'] }
        ]
    },
    plugins: [
        //new webpack.optimize.LimitChunkCountPlugin({ maxChunks: 1 }),
        new HtmlWebpackPlugin({ title: "Moiety" })
    ]
}
