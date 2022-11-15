import { FermiClient } from 'fermi-js-sdk'

type StoredClients = Record<string, FermiClient>

const storedClients: StoredClients = {}

export function getFermiClient({
  jsonRpcUrl,
  useStored = false,
}: {
  jsonRpcUrl: string
  useStored?: boolean
}) {
  if (useStored && storedClients[jsonRpcUrl]) {
    return storedClients[jsonRpcUrl]
  }

  const client = new FermiClient(jsonRpcUrl)
  storedClients[jsonRpcUrl] = client
  return client
}
