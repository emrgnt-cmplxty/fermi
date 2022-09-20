import useMangoStore from '../stores/useMangoStore'
import { MangoAccount } from '@gdexorg/ifm-client'
import shallow from 'zustand/shallow'

export default function useMangoAccount(): {
  mangoAccount: MangoAccount | null
  initialLoad: boolean
} {
  const { mangoAccount, initialLoad } = useMangoStore(
    (state) => ({
      mangoAccount: state.selectedMangoAccount.current,
      lastUpdatedAt: state.selectedMangoAccount.lastUpdatedAt,
      initialLoad: state.selectedMangoAccount.initialLoad,
    }),
    shallow
  )

  return { mangoAccount, initialLoad }
}
