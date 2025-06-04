const { spawnSync }  = require("child_process");
const { existsSync } = require("fs")           ;
const { join }       = require("path")         ;

const PLATFORM = process.platform; // "aix"   | "darwin"  | "freebsd" | "linux"   | "openbsd" | "sunos"  | "win32"
const ARCH     = process.arch    ; // "arm"   | "arm64"   | "ia32"    | "loong64" | "mips"    | "mipsel" | "ppc"
                                   // "ppc64" | "riscv64" | "s390"    | "s390x"   | "x64"

let LIB = "gnu";
if (PLATFORM === "linux") {
  const { stdout, stderr } = spawnSync("ldd", ["--version"], { encoding: "utf8" });
  if ((stdout + stderr).toLowerCase().includes("musl"))
    LIB = "musl";
}

let BASE_NAME;
if      (PLATFORM === "linux") BASE_NAME = `linux-${ARCH}-${LIB}`;
else if (PLATFORM === "win32") BASE_NAME = `win32-${ARCH}-msvc`;
else throw new Error(`Unsupported platform: ${PLATFORM}`);


const BINDING_PATH = join(__dirname, "dist", `${BASE_NAME}.node`);
if (!existsSync(BINDING_PATH))
  throw new Error(`The native binary for your platform/architecture was not found.\nSearched in: ${BINDING_PATH}`);

module.exports = require(BINDING_PATH);