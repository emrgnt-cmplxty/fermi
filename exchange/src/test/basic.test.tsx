import { renderHook } from '@testing-library/react-hooks/pure'
import { useTransactions } from 'hooks/react-query/transaction'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

jest.setTimeout(30000)
jest.mock('utils/env', () => {
  return {
    isProd: jest.fn(() => false),
    isTestnet: jest.fn(() => true),
  }
})

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      // ✅ turns retries off
      retry: false,
    },
  },
})
const wrapper: React.FC = ({ children }) => (
  <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
)

describe('react-query', () => {
  // it('useDexPriceInfo', async () => {
  //   const { result, waitFor } = renderHook(
  //     () => useDexPriceInfo(undefined, false),
  //     {
  //       wrapper,
  //     },
  //   )
  //   await waitFor(() => result.current.isSuccess)
  //   // console.log(result.current.data)
  // })
  // it('useServerOnlyTokensMap', async () => {
  //   const { result, waitFor } = renderHook(() => useServerOnlyTokensMap(), {
  //     wrapper,
  //   })
  //   await waitFor(() => result.current.isSuccess)
  //   // console.log(result.current.data)
  // })
  // it('useAlgofiClientData', async () => {
  //   const { result, waitFor } = renderHook(() => useAlgofiClientData(undefined), {
  //     wrapper,
  //   })
  //   await waitFor(
  //     () => {
  //       return result.current.isSuccess
  //     },
  //     { timeout: 30000 },
  //   )
  //   console.log(result.current.data)
  // })

  // it('useFullMarketData', async () => {
  //   await act(async () => {
  //     const { result, waitFor } = renderHook(
  //       () => useFullMarketData({ node: '' }),
  //       {
  //         wrapper,
  //       },
  //     )
  //     await waitFor(
  //       () => {
  //         return !result.current.isLoading
  //       },
  //       { timeout: 30000 },
  //     )

  //     console.log(result.current.data)
  //   })
  // })

  it('useTransactions', async () => {
    const { result, waitFor } = renderHook(
      () =>
        useTransactions(
          'WO7ZA3GTZZTWEIU427ZZZ674RPYUUN5LOQJF5TBY2YYFBTTNOR33X45RYI',
        ),
      {
        wrapper,
      },
    )
    await waitFor(
      () => {
        return result.current.isSuccess
      },
      { timeout: 30000 },
    )
  })
})
