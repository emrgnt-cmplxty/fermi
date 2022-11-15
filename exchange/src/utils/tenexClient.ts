import { TenexClient } from 'tenex-js-sdk'

type StoredClients = Record<string, TenexClient>

const storedClients: StoredClients = {}

export function getTenexClient({
  jsonRpcUrl,
  useStored = false,
}: {
  jsonRpcUrl: string
  useStored?: boolean
}) {
  if (useStored && storedClients[jsonRpcUrl]) {
    return storedClients[jsonRpcUrl]
  }

  const client = new TenexClient(jsonRpcUrl)
  storedClients[jsonRpcUrl] = client
  return client
}
