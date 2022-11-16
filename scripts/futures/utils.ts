export function getJsonRpcUrl(authority): string {
    // construct the URL dynamically from the multiaddr
    let link = authority['jsonrpc_address'].split('/')[2]
    let port = authority['jsonrpc_address'].split('/')[4]
    let url = "http://" + link + ":" + port
    return url
}