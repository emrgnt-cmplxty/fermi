import { createSyncStoragePersister } from '@tanstack/query-sync-storage-persister'
import {
  MutationCache,
  QueryCache,
  QueryClient,
  QueryClientProvider,
} from '@tanstack/react-query'
import {
  PersistedClient,
  persistQueryClient,
} from '@tanstack/react-query-persist-client'
import { useSnackbar } from 'notistack'
import { ReactElement, ReactNode, useMemo } from 'react'

interface QueryProviderProps {
  children: ReactNode
}

function QueryProvider({ children }: QueryProviderProps): ReactElement {
  const { enqueueSnackbar } = useSnackbar()

  const queryClient = useMemo(() => {
    const client = new QueryClient({
      defaultOptions: {
        cacheTime: 1000 * 60 * 60 * 24,
        queries: {
          refetchOnWindowFocus: false,
        },
      } as any,

      // queryCache/mutationCache onError will always be called on every error
      // to default with option override locally, use defaultOptions.queries.onError
      queryCache: new QueryCache({
        // done globally so only triggered once per query
        onError: (error, query) => {
          // show error toasts if we already have data in the cache
          // which indicates a failed background update
          if (query.state.data !== undefined) {
            let errorMessage = `Query error: `
            if (error instanceof Error) {
              errorMessage += error.message
            }
            console.error(errorMessage, query)
            //suppress failed to fetch
            if (errorMessage !== 'Query Error: Failed to fetch') {
              enqueueSnackbar(errorMessage)
            }
          }
        },
      }),
      mutationCache: new MutationCache({
        onError: (error, _variables, _context, mutation) => {
          let errorMessage = 'Mutation error: '
          if (error instanceof Error) {
            errorMessage += error.message
          }
          console.error(errorMessage)
        },
      }),
    })

    const localStoragePersistor = createSyncStoragePersister({
      storage: window.localStorage,
      serialize: (client: PersistedClient) => {
        const duplicateState = Object.assign({}, client)
        // duplicateState.clientState.queries =
        //   duplicateState.clientState.queries.filter(({ queryKey }: any) => {
        //     return (
        //       Array.isArray(queryKey) &&
        //       queryKey.filter((key) => key === 'userData').length > 0
        //     )
        //   })
        return JSON.stringify(duplicateState)
      },
    })

    persistQueryClient({
      queryClient: client,
      persister: localStoragePersistor,
    })

    return client
  }, [enqueueSnackbar])

  return (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  )
}

export default QueryProvider
