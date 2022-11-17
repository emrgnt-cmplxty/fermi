"use strict";
exports.__esModule = true;
exports.getJsonRpcUrl = void 0;
function getJsonRpcUrl(authority) {
    // construct the URL dynamically from the multiaddr
    var link = authority['jsonrpc_address'].split('/')[2];
    var port = authority['jsonrpc_address'].split('/')[4];
    var url = "http://" + link + ":" + port;
    return url;
}
exports.getJsonRpcUrl = getJsonRpcUrl;
