import { me } from "../index.macro" with { types: "macro" };
import { do_not_optimize, summary, bench, run } from "mitata";
import { Method, Rest } from "kodkord";
import { RST } from "../index";

const c = { authorization: `Bot ${Bun.env.TOKEN}`, user_agent: "Cement/alpha" };

const n = new RST(c);
const o = new Rest(c);

summary(() => {
    bench("new", async () => {
        do_not_optimize(await n.get(me()));
    });
    
    bench("NEW (dispatch)", () => {
        do_not_optimize(n.dispatch({
            method: "GET",
            route: me(),
        }));
    });
    
    bench("OLD", async () => {
        o.start_scheduler();
        
        do_not_optimize(await o.request({
            method: Method.GET,
            route: "v10/users/@me"
        }));

        o.stop_scheduler();
    });
});

run();