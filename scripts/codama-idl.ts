import { createFromRoot } from "codama";
import { rootNodeFromAnchor } from "@codama/nodes-from-anchor";
import fs from "fs";
const anchorIdl = require("../smart-contract/idl/access_protocol.json");
const node = rootNodeFromAnchor(anchorIdl)
const codama = createFromRoot(node)
const output = codama.getJson()
fs.writeFileSync("codama-idl.json", output)