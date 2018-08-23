const HtmlPlugin = require("html-webpack-plugin");

module.exports = {
    mode: "production",
    plugins: [
        new HtmlPlugin({template: "src/index.html"}),
    ],
};