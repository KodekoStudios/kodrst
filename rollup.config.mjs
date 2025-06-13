import typescript from "@rollup/plugin-typescript";
import resolve from "@rollup/plugin-node-resolve";
import dts from "rollup-plugin-dts";

/** @type {import("rollup").RollupOptions[]} */
export default [{
    // Js Bundle
    input: "src/index.ts",
    output: {
        file: "lib/index.js",
        format: "esm",
        sourcemap: false
    },
    plugins: [
        resolve(),
        typescript({
            tsconfig: "./tsconfig.json",
            declaration: false,
            sourceMap: false,
            outDir: undefined
        }),
    ],
}, {
    // d.ts bundle
    input: "src/index.ts",
    output: {
        file: "lib/index.d.ts",
        format: "esm",
    },
    plugins: [dts()],
}]