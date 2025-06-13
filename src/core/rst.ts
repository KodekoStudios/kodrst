import { type Pointer, type FFIType, JSCallback, CString, ptr } from "bun:ffi";
import { symbols, cstring } from "../symbols";
import { Response } from "./response";

const registry = new FinalizationRegistry((ptr: Pointer) => symbols.destructor_rst(ptr));

export interface File {
    content_type: string
    field       : string
    name        : string
    buffer      : Uint8Array
}

export class RST {
    #ptr: Pointer;

    public constructor(authorization: string, user_agent: string) {
        this.#ptr = symbols.constructor_rst(cstring(authorization), cstring(user_agent))!;
        if (this.#ptr === null) throw new Error("Failed to build RST");

        registry.register(this, this.#ptr);
    }

    public send(method: string, route: string, body?: string, files?: File[], reason?: string): Promise<Response> {
        let files_ptr: Pointer | null = null;
        
        if (files && files.length != 0) {
            const len = files.length;
            files_ptr = symbols.alloc_file_slice(len);

            for (let i = 0; i < len; i++) {
                const { buffer, content_type, field, name } = files[i]!;

                symbols.alloc_file(
                    files_ptr, 
                    buffer, buffer.length,
                    cstring(content_type),
                    cstring(field       ),
                    cstring(name        ),
                );
            }
        }

        const request = symbols.constructor_request(
            cstring(method),
            cstring(route) ,
            files_ptr, files?.length ?? 0  ,
            body   ? cstring(body)   : null, 
            reason ? cstring(reason) : null
        );

        return new Promise<Response>((resolve, reject) => {
            const resolver = new JSCallback((ptr: Pointer) => {
                resolve(new Response(ptr));
                
                symbols.dealloc_file_slice(files_ptr, files?.length ?? 0);
                symbols.destructor_request(request);
                resolver.close();
            }, {
                args: ["pointer"],
                returns: "void",
                threadsafe: true
            });

            const rejecter = new JSCallback((ptr: Pointer, len: FFIType.u32) => {
                reject(new CString(ptr, 0, len));
                
                symbols.dealloc_file_slice(files_ptr, files?.length ?? 0);
                symbols.destructor_request(request);
                rejecter.close();
            }, {
                args: ["pointer"],
                returns: "void",
                threadsafe: true
            });

            symbols.send_rst(
                resolver.ptr,
                rejecter.ptr,
                this.#ptr,
                request,
            );
        });
    }

    public __destroy__() {
        symbols.destructor_rst(this.#ptr);
        registry.unregister(this);
    }
}