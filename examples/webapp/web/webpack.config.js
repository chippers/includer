const HtmlPlugin = require("html-webpack-plugin");

module.exports = {
    mode: "production",
    output: {
        filename: "[chunkhash].js",
        publicPath: "/",
    },
    plugins: [
        new HtmlPlugin({template: "src/index.html"}),
    ],
};