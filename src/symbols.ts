import { dlopen } from "bun:ffi";

import { resolve, join } from "node:path";

function artifact_path() {
    if (Bun.env.KODRST_ARTIFACT) return Bun.env.KODRST_ARTIFACT;

    if (process.platform === "linux") {
        switch (process.arch) {
            case "riscv64": return resolve(join("../artifacts/", "riscv64-linux-gnu.so"));
            case "arm64"  : return resolve(join("../artifacts/", "aarch64-linux-gnu.so"));
            case "x64"    : return resolve(join("../artifacts/",     "x64-linux-gnu.so"));
            case "arm"    : return Bun.env.ARM_VERSION !== "v7" 
                                        ? resolve(join("../artifacts/",   "arm-linux-gnu.so")) 
                                        : resolve(join("../artifacts/", "armv7-linux-gnu.so"));
        }
    }

    throw new Error(
        `❌ No prebuilt artifact found for your platform (${process.platform}) and architecture (${process.arch}).
        
Kodrst currently does not ship precompiled artifacts for this combination.

✅ To fix this, you can:

  1. Clone the repository:
     git clone https://github.com/KodekoStudios/kodrst.git

  2. Install dependencies:
     bun install

  3. Compile the native artifact for your platform:
     cargo zigbuild --release | cargo build --release

  4. Then, specify the path to the generated artifact by setting the environment variable:
     KODRST_ARTIFACT=/absolute/path/to/your/artifact.so

💡 Example:
     KODRST_ARTIFACT=~/artifacts/custom-linux-xyz.so bun your-app.ts

If you're using an unsupported OS or architecture, feel free to open an issue or contribute support.

`
    );
}

export const { symbols } = dlopen(artifact_path(), {
    constructor_rst: {
        args: ["pointer", "pointer"],
        returns: "pointer"
    } as const,

    destructor_rst: {
        args: ["pointer"],
        returns: "void"
    } as const,

    send_rst: {
        args: ["function", "function", "pointer", "pointer"],
        returns: "void"
    } as const,

    constructor_request: {
        args: ["pointer", "pointer", "pointer", "usize", "pointer", "pointer"],
        returns: "pointer"
    } as const,

    destructor_request: {
        args: ["pointer"],
        returns: "void"
    } as const,

    headers_len: {
        args: ["pointer"],
        returns: "u8"
    } as const,

    headers: {
        args: ["pointer"],
        returns: "pointer"
    } as const,

    destructor_response: {
        args: ["pointer"],
        returns: "void"
    } as const,
    
    alloc_file_slice: {
        args: ["usize"],
        returns: "pointer"
    } as const,

    dealloc_file_slice: {
        args: ["pointer", "usize"],
        returns: "void"
    } as const,

    alloc_file: {
        args: ["pointer", "buffer", "usize", "pointer", "pointer", "pointer"],
        returns: "pointer",
    } as const,
});

export function cstring(str: string): Uint8Array {
    return new TextEncoder().encode(str.concat("\0"));
}