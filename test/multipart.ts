import { channel_messages } from "../index.macro" with { types: "macro" };
import { RST } from "../index";

const rst = new RST({
    authorization: `Bot ${Bun.env.TOKEN}`,
    user_agent   : "Cement/alpha"
});


let res = await rst.send({
    route: channel_messages("913243741710594048"),
    body: JSON.stringify({
        content: "hi",
        attachments: [{
            filename: "sample_image.jpg",
            id: 0,
        }]
    }, null, 2),
    files: [{
        content_type: "image/jpg",
        field: "files[0]",
        name: "sample_image.jpg",
        data: await Bun.file(Bun.resolveSync("./sample_image.jpg", __dirname)).bytes()
    }],
    method: "POST",
});

console.log(res.body_as_str());