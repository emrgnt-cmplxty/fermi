const getJsonRpcUrl = (authority: any) => {
  // construct the URL dynamically from the multiaddr
  var [_, _, link, _, port] = authority['jsonrpc_address'].split('/')
  var url = `http://${link}:${port}`
  return url
}

export default getJsonRpcUrl
