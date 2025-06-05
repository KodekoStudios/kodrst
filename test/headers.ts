import { me } from "../index.macro" with { types: "macro" };
import { RST } from "../index";

const rst = new RST({
    authorization: `Bot ${Bun.env.TOKEN}`,
    user_agent   : "Cement/alpha"
});


let res = await rst.get(me());

console.log(res.headers);