export const enum Method {
    DELETE = "DELETE",
    PATCH  = "PATCH" ,
    POST   = "POST"  ,
    PUT    = "PUT"   ,
    GET    = "GET"   ,
}

export interface File {
    content_type: string    ;
    field       : string    ;
    name        : string    ;
    data        : Uint8Array;
}

export interface Request {
    method : Method | "DELETE" | "PATCH" | "POST"
                    | "PUT"    | "GET"   ,
    route  : string,
    reason?: string,
    files ?: File[],
    body  ?: string,
}

export interface Settings {
    authorization: string,
    user_agent   : string,
}

export declare class RST {
    public constructor(settings: Settings);

    public dispatch(request: Request): undefined;
    public send    (request: Request): Promise<Response>;

    public delete(route: string, body?: string, files?: File[], reason?: string): Promise<Response>;
    public patch (route: string, body?: string, files?: File[], reason?: string): Promise<Response>;
    public post  (route: string, body?: string, files?: File[], reason?: string): Promise<Response>;
    public put   (route: string, body?: string, files?: File[], reason?: string): Promise<Response>;
    public get   (route: string, body?: string, files?: File[], reason?: string): Promise<Response>;
}

export declare class Response {
    private constructor();
    public get headers (): Record<string, string>;
    public body_as_json(): Record<string, string>;
    public body_as_str (): string;
}