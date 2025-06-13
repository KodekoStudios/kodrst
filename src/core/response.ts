import { CString, type Pointer, read, toArrayBuffer } from "bun:ffi";
import { symbols } from "../symbols";

const registry = new FinalizationRegistry((ptr: Pointer) => symbols.destructor_response(ptr));

export class Response {
    #ptr: Pointer;

    public constructor(ptr: Pointer) {
        registry.register(this, this.#ptr = ptr);
    }

    public status(): number {
        return read.u16(this.#ptr, 0x20);
    }

    public headers(): Record<string, string> {
        const headers: Record<string, string> = {};
        const ptr = read.ptr(this.#ptr, 0x8) as Pointer;
        const len = read.u8(this.#ptr);

        for (let i = 0; i < len * 0x10; i += 0x10) {
            // @ts-expect-error
            headers[new CString(read.ptr(ptr, i) as Pointer)] = new CString(read.ptr(ptr, i + 0x8) as Pointer);
        }

        return headers;
    }

    public body(): CString {
        return new CString(read.ptr(this.#ptr, 0x10) as Pointer);
    }

    public __destroy__() {
        symbols.destructor_response(this.#ptr);
        registry.unregister(this);
    }
}