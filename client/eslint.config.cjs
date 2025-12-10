const {
    defineConfig,
} = require("eslint/config");

const globals = require("globals");
const parser = require("vue-eslint-parser");
const js = require("@eslint/js");

const {
    FlatCompat,
} = require("@eslint/eslintrc");

const compat = new FlatCompat({
    baseDirectory: __dirname,
    recommendedConfig: js.configs.recommended,
    allConfig: js.configs.all
});

module.exports = defineConfig([{
    languageOptions: {
        globals: {
            ...globals.browser,
        },

        parser: parser,
        ecmaVersion: 2021,
        sourceType: "module",

        parserOptions: {
            parser: "@typescript-eslint/parser",
        },
    },

    extends: compat.extends(
        "eslint:recommended",
        "plugin:@typescript-eslint/recommended",
        "plugin:prettier/recommended",
    ),

    rules: {
        "@typescript-eslint/no-unused-vars": ["warn"],
    },
}]);