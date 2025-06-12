import { type Pointer, read, toArrayBuffer } from "bun:ffi";

import { symbols } from "./symbols";

const registry = new FinalizationRegistry((ptr: Pointer) => symbols.destructor_response(ptr));

export class Response {
    #ptr: Pointer;

    public constructor(ptr: Pointer) {
        registry.register(this, this.#ptr = ptr);
    }

    public status(): number {
        return read.u16(this.#ptr, 0x20);
    }

    public headers() {
        // const headers: Record<string, string> = {};

        // const len   = symbols.headers_len(this.#ptr);
        // const slice = symbols.headers    (this.#ptr);

        // for (let i = 0; i < len; i++) {
        //     const offset = i * 0x20;

        //     let name  = read.u8(slice!, offset);
        //     let value = read.ptr(slice!, offset + 1);

        //     console.log(name, toArrayBuffer(Pointer(String(value))));
        // }
    }

    public __destroy__() {
        symbols.destructor_response(this.#ptr);
        registry.unregister(this);
    }
}