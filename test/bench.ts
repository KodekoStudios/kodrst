// Benchmark notes:
//
// While `NEW (send)` and `NEW (get)` appear slower than `OLD (req)` and `djs`, the context matters:
// - `OLD (req)` runs in ~420ms but consumes around 760KB of memory on average.
// - `djs` (discord.js/rest) is about twice as fast (~230ms) and uses only ~115KB,
//   but it **does not implement any rate limit protection or exponential backoff mechanisms**.
// - In contrast, `NEW (send)` *does* include full rate limit handling and still achieves ~500ms
//   with just ~114KB of memory usage.
// - Moreover, when using specific HTTP methods (`get`, `patch`, `delete`, etc.) instead of `send`,
//   performance improves and memory usage drops to around ~40KB.
//
// `dispatch` is considered separately as it obliterates the others in performance.
// It is specifically optimized to **not return**, making it a fast-path design.
//
// All benchmarks include full REST client construction on each iteration,
// so the times reflect both setup and execution.


import { me } from "../index.macro" with { types: "macro" };

import { do_not_optimize, barplot, bench, run } from "mitata";
import { Method, Rest } from "kodkord";
import { REST } from "@discordjs/rest";
import { RST } from "../index";

const c = { authorization: `Bot ${Bun.env.TOKEN}`, user_agent: "Cement/alpha" };

barplot(() => {
    bench("new (send)", async () => {
        const n = new RST(c);

        do_not_optimize(await n.send({
            method: "GET",
            route: me(),
        }));
    });

    bench("new (get)", async () => {
        const n = new RST(c);

        do_not_optimize(await n.get(me()));
    });
    
    // This js piss off the others
    // bench("NEW (dispatch)", () => {
    //     const n = new RST(c);

    //     do_not_optimize(n.dispatch({
    //         method: "GET",
    //         route: me(),
    //     }));
    // });
    
    bench("OLD (req)", async () => {
        const o = new Rest(c);
        o.start_scheduler();
        
        do_not_optimize(await o.request({
            method: Method.GET,
            route: "v10/users/@me"
        }));

        o.stop_scheduler();
    });

    bench("djs", async() => {
        const d = new REST({ userAgentAppendix: c.user_agent }).setToken(Bun.env.TOKEN!);

        do_not_optimize(await d.request({
            // @ts-expect-error
            method: "GET",
            fullRoute: "/users/@me"
        }));
    });
});

run();